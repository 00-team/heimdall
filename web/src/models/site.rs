use std::{collections::HashMap, future::Future, pin::Pin};

use super::{AppErr, AppErrBadRequest, JsonStr};
use crate::{config::Config, models::AppErrNotFound, AppState};
use actix_http::Payload;
use actix_web::{
    web::{Data, Path},
    FromRequest, HttpRequest,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema, Clone, Debug)]
pub struct Status {
    pub code: u16,
    pub count: u64,
    pub max_time: u64,
    pub min_time: u64,
    pub total_time: u64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone, Default)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub timestamp: i64,
    pub latest_request: i64,
    pub latest_ping: i64,
    pub total_requests: i64,
    pub total_requests_time: i64,
    pub requests_max_time: i64,
    pub requests_min_time: i64,
    #[schema(value_type = HashMap<String, u64>)]
    pub status: JsonStr<HashMap<String, Status>>,
    pub token: Option<String>,
    pub online: bool,
    pub latest_message_timestamp: i64,
    pub latest_dump_timestamp: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone, Default)]
pub struct SiteMessage {
    pub id: i64,
    pub site: i64,
    pub timestamp: i64,
    pub text: String,
    pub tag: String,
}

impl Site {
    pub fn verify_name(name: &str) -> Result<(), AppErr> {
        if name.is_empty() || name.len() > 100 {
            return Err(AppErrBadRequest("invalid name length > 0 && < 100"));
        }

        if !name.chars().all(|c| Config::SITE_NAME_ABC.contains(&(c as u8))) {
            return Err(AppErrBadRequest("invalid name characters"));
        }

        Ok(())
    }
}

impl FromRequest for Site {
    type Error = AppErr;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        #[derive(Deserialize)]
        struct Sid {
            site_id: i64,
        }
        let state = req.app_data::<Data<AppState>>().unwrap().clone();
        let path = Path::<Sid>::extract(req);

        Box::pin(async move {
            let sites = state.sites.lock().await;
            if let Some(site) = sites.get(&path.await?.site_id) {
                Ok(site.clone())
            } else {
                Err(AppErrNotFound("no site was found"))
            }
        })
    }
}
