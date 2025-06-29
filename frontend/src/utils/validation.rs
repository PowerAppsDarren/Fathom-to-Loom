use regex::Regex;

pub fn is_valid_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}

pub fn is_strong_password(password: &str) -> bool {
    password.len() >= 8 &&
    password.chars().any(|c| c.is_uppercase()) &&
    password.chars().any(|c| c.is_lowercase()) &&
    password.chars().any(|c| c.is_numeric())
}

pub fn validate_api_key(key: &str) -> bool {
    !key.trim().is_empty() && key.len() >= 10
}
