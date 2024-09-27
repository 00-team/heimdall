use std::{future::Future, pin::Pin};

use actix_http::Payload;
use actix_web::{
    web::{Data, Path},
    FromRequest, HttpRequest,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{config::Config, AppState};

use super::{
    inner_deref, user::Authorization, AppErr, AppErrBadRequest, AppErrForbidden,
};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema, Clone)]
pub struct Site {
    pub id: i64,
    pub name: String,
    pub latest_request: i64,
    pub total_requests: i64,
    pub token: Option<String>,
}

impl Site {
    pub fn verify_slug(slug: &str) -> Result<(), AppErr> {
        if slug.len() < 1 || slug.len() > 100 {
            return Err(AppErrBadRequest("invalid slug length > 0 && < 100"));
        }

        if !slug.chars().all(|c| Config::SLUG_ABC.contains(&(c as u8))) {
            return Err(AppErrBadRequest("invalid slug characters"));
        }

        Ok(())
    }
}

impl FromRequest for Site {
    type Error = AppErr;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        #[derive(Deserialize)]
        struct PP {
            site_slug: String,
        }
        let state = req.app_data::<Data<AppState>>().unwrap();
        let pool = state.sql.clone();
        let path = Path::<PP>::extract(req);

        Box::pin(async move {
            let path = path.await?;
            let result = sqlx::query_as! {
                Site,
                "select * from sites where slug = ?",
                path.site_slug
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
            if let Authorization::Site { slug, token } = auth? {
                let result = sqlx::query_as! {
                    Site,
                    "select * from sites where slug = ? AND token = ?",
                    slug, token
                }
                .fetch_one(&pool)
                .await?;

                return Ok(SiteAuth(result));
            }

            Err(AppErrForbidden("invalid site auth"))
        })
    }
}
