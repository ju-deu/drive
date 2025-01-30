use axum::http::Method;
use axum::routing::{get, post, put};
use axum::{middleware, Extension, Router};
use axum_extra::extract::cookie::Key;
use dotenv::dotenv;
use drive_lib::handlers::files::upload::upload;
use drive_lib::handlers::users::authenticate::auth;
use drive_lib::handlers::users::login::login;
use drive_lib::handlers::users::new::new;
use drive_lib::handlers::users::refresh::refresh_token;
use drive_lib::handlers::users::update;
use drive_lib::models::appstate::{Appstate, AppstateWrapper};
use sqlx::PgPool;
use std::env;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    // load env
    dotenv().ok();

    let jwt_secret = env::var("JWT_SECRET").unwrap();
    let cookie_secret = env::var("COOKIE_SECRET").unwrap();
    let file_location = env::var("FILE_LOCATION").unwrap();
    let psql_url = env::var("DATABASE_URL").unwrap();

    // db connection
    let pool = PgPool::connect(&psql_url).await.unwrap();
    let shared_pool = Arc::new(pool);


    // appstate
    let appstate = Arc::new(Appstate::new(
        shared_pool,
        jwt_secret,
        Key::try_from(cookie_secret.as_bytes()).unwrap(),
        file_location
    ));
    let wrapped_appstate = AppstateWrapper(appstate);

    // set up http server
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        /*.allow_headers([
            "content-type".parse().unwrap(),
            "authorization".parse().unwrap(),
            "cookie".parse().unwrap(),
            "host".parse().unwrap(),
            "user-agent".parse().unwrap(),
        ])
        .allow_credentials(true)*/;

    // axum
    let protected_file_routes = Router::new()
        .route("/upload", post(upload))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(auth))
                .layer(Extension(wrapped_appstate.clone()))
        );



    let protected_user_routes = Router::new()
        .route("/password/change", put(update::password::change::change_password))
        .route("/username/change", put(update::username::change::change_username))
        .route("/refresh_token", get(refresh_token))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(auth))
                .layer(Extension(wrapped_appstate.clone()))
        );

    let public_user_routes = Router::new()
        .route("/new", post(new))
        .route("/login", post(login));



    // set up axum
    let app = Router::new()
        .nest("/v1/file", protected_file_routes)
        .nest("/v1/user", protected_user_routes)
        .nest("/v1/user", public_user_routes)
        .layer(
            ServiceBuilder::new()
                .layer(Extension(wrapped_appstate.clone()))
                .layer(TraceLayer::new_for_http())
                .layer(cors)
        )
        .with_state(wrapped_appstate.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
