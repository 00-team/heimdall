use std::sync::OnceLock;

#[derive(Debug)]
/// Main Config
pub struct Config {}

macro_rules! evar {
    ($name:literal) => {
        std::env::var($name).expect(concat!($name, " was not in .env"))
    };
}
pub(crate) use evar;

impl Config {
    pub const RECORD_DIR: &'static str = "record";
    pub const CODE_ABC: &'static [u8] = b"0123456789";
    pub const TOKEN_ABC: &'static [u8] =
        b"!@#$%^&*_+abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*_+";
    pub const SITE_NAME_ABC: &'static [u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.";
}

pub fn config() -> &'static Config {
    static STATE: OnceLock<Config> = OnceLock::new();
    STATE.get_or_init(|| Config {})
}
