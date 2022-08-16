use config::{Config, ConfigError};

pub fn resolve_string(settings: &Config, key: &str, default: &str) -> String {
    match settings.get_string(key) {
        Ok(res) => res,
        _ => String::from(default)
    }
}

pub fn resolve_bool(settings: &Config, key: &str, default: bool) -> bool {
    match settings.get_bool(key) {
        Ok(res) => res,
        Err(_) => default
    }
}