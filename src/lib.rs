

pub mod handlers {
    pub mod users {
        pub mod update {
            pub mod username {
                pub mod change;
            }
            pub mod password {
                pub mod change;
            }
        }
        pub mod authenticate;
        pub mod login;
        pub mod refresh;
        pub mod new;
    }
    pub mod files {
        pub mod download;
        pub mod delete;
        pub mod upload;
    }
}

pub mod models {
    pub mod user;
    pub mod appstate;
    pub mod file;
}

pub mod util {
    pub mod jwt {
        pub mod claims;
    }
    pub mod validation;
}