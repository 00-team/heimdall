use std::{collections::HashMap, future::Future, pin::Pin};

use actix_http::Payload;
use actix_web::{
    web::{Data, Path},
    FromRequest, HttpRequest,
};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use utoipa::ToSchema;

use crate::{config::Config, AppState};

use super::{
    inner_deref, user::Authorization, AppErr, AppErrBadRequest,
    AppErrForbidden, JsonStr,
};

#[derive(Debug, Serialize, Deserialize, FromRow, ToSchema, Clone, Default)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub latest_request: i64,
    pub latest_ping: i64,
    pub total_requests: i64,
    pub total_requests_time: i64,
    pub total_requests_size: i64,
    pub status: JsonStr<HashMap<u16, u64>>,
    pub token: Option<String>,
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
        if name.len() < 1 || name.len() > 100 {
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
        struct SID {
            site_id: i64,
        }
        let state = req.app_data::<Data<AppState>>().unwrap();
        let pool = state.sql.clone();
        let path = Path::<SID>::extract(req);

        Box::pin(async move {
            let path = path.await?;
            let result = sqlx::query_as! {
                Site,
                "select * from sites where id = ?",
                path.site_id
            }
            .fetch_one(&pool)
            .await?;

            Ok(result)
        })
    }
}

pub struct SiteAuth(pub Site);
inner_deref!(SiteAuth, Site);

impl FromRequest for SiteAuth {
    type Error = AppErr;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        let state = req.app_data::<Data<AppState>>().unwrap();
        let pool = state.sql.clone();
        let auth = Authorization::try_from(req);
        // let path = Path::<PP>::extract(req);

        Box::pin(async move {
            if let Authorization::Site { id, token } = auth? {
                let result = sqlx::query_as! {
                    Site,
                    "select * from sites where id = ? AND token = ?",
                    id, token
                }
                .fetch_one(&pool)
                .await?;

                return Ok(SiteAuth(result));
            }

            Err(AppErrForbidden("invalid site auth"))
        })
    }
}
