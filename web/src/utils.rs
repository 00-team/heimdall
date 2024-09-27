use crate::config::Config;
use crate::models::{AppErr, AppErrBadRequest};
use rand::Rng;

pub fn phone_validator(phone: &str) -> Result<(), AppErr> {
    if phone.len() != 11 || !phone.starts_with("09") {
        return Err(AppErrBadRequest("invalid phone number"));
    }

    if phone.chars().any(|c| !c.is_ascii_digit()) {
        return Err(AppErrBadRequest("phone number must be all digits"));
    }

    Ok(())
}

pub fn now() -> i64 {
    chrono::Local::now().timestamp()
}

pub fn get_random_string(charset: &[u8], len: usize) -> String {
    let mut rng = rand::thread_rng();
    (0..len).map(|_| charset[rng.gen_range(0..charset.len())] as char).collect()
}

pub trait CutOff {
    fn cut_off(&mut self, len: usize);
}

impl CutOff for String {
    fn cut_off(&mut self, len: usize) {
        let mut idx = len;
        loop {
            if self.is_char_boundary(idx) {
                break;
            }
            idx -= 1;
        }
        self.truncate(idx)
    }
}

impl CutOff for Option<String> {
    fn cut_off(&mut self, len: usize) {
        if let Some(v) = self {
            v.cut_off(len)
        }
    }
}
