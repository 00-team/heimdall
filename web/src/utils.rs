use crate::models::{AppErr, AppErrBadRequest};
use rand::Rng;
use serde::Serialize;
use crate::config;

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

pub async fn send_message(text: &str) {
    if cfg!(debug_assertions) {
        log::info!("send_message: {text}");
        return;
    }

    let client = awc::Client::new();
    let conf = config::config();
    let url = format!(
        "https://api.telegram.org/bot{}/sendMessage?chat_id={}&message_thread_id=438",
        conf.bot_token, conf.group_id
    );
    let request = client.post(&url);

    #[derive(Serialize, Debug)]
    struct Body {
        text: String,
    }

    let _ = request.send_json(&Body { text: text.to_string() }).await;
    // match result {
    //     Ok(mut v) => {
    //         log::info!("topic: {}", topic);
    //         log::info!("text: {}", text);
    //         log::info!("send message status: {:?}", v.status());
    //         log::info!("send message: {:?}", v.body().await);
    //     }
    //     Err(e) => {
    //         log::info!("send message err: {:?}", e);
    //     }
    // }
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
