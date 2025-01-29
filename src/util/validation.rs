


/// Username validation with following requirements:
/// - 4-16 chars of length
/// - only a-z, A-Z, 0-9, ., -, _
pub fn username(username: &String) -> (bool, String) {
    // check for length in bounds of 4-16
    if username.len() < 4 || username.len() > 16 {
        return (false, "Length of username not in bounds of 4-16".to_string())
    }

    // check for illegal chars
    let allowed = username.chars().all(|c| {
        matches!(c,
            'a'..='z' |
            'A'..='Z' |
            '0'..='9' |
            '.' | '-' | '_'
        )
    });

    if !allowed {
        return (false, "Contains non a-z, A-Z, ., _, - char".to_string())
    }

    (true, "".to_string())
}

/// Password validation with following requirements:
/// - 8-30 chars of length
/// - only a-z, A-Z, 0-9, ., _, -, *, #, %, &, $, ?,
/// - at least 1 of each listed above
pub fn password(password: &String) -> (bool, String) {
    // check for length
    if password.len() < 8 || password.len() > 30 {
        return (false, "Length of password is not in bounds of 8-30".to_string())
    }

    // check for allowed chars
    let allowed = password.chars().all(|c| {
        matches!(c,
            'a'..='z' |
            'A'..='Z' |
            '0'..='9' |
            '.' | '-' | '_' | '*' | '#' | '%' | '&' | '$' | '?'
        )
    });
    if !allowed {
        return (false, "Includes disallowed char(s)".to_string())
    }


    // check for each req
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return (false, "Does not include lowercase char".to_string())
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return (false, "Does not include uppercase char".to_string())
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return (false, "Does not include digit".to_string())
    }
    if !password.chars().any(|c| { vec!['.', '_', '-', '*', '#', '%', '&', '$', '?'].contains(&c) }) {
        return (false, "Does not include special char".to_string())
    }


    (true, "".to_string())
}