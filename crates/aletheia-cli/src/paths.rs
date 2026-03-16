use std::path::PathBuf;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aletheia")
}

pub fn keys_dir() -> PathBuf {
    config_dir().join("keys")
}

pub fn sessions_dir() -> PathBuf {
    config_dir().join("sessions")
}

pub fn session_dir(name: &str) -> PathBuf {
    sessions_dir().join(name)
}
