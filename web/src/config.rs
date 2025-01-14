#[derive(Debug)]
/// Main Config
pub struct Config {
    pub bot_token: String,
    pub group_id: String,
}

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
    pub const SITE_TOKEN_ABC: &'static [u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHMNOPQRSTUVWXYZ0123456789";
    pub const SITE_NAME_ABC: &'static [u8] =
        b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_.";
}

use std::sync::OnceLock;
pub fn config() -> &'static Config {
    static STATE: OnceLock<Config> = OnceLock::new();
    STATE.get_or_init(|| Config {
        bot_token: evar!("TELOXIDE_TOKEN"),
        group_id: evar!("TELOXIDE_GROUP_ID"),
    })
}
