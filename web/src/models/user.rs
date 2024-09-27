use super::{inner_deref, AppErr, AppErrForbidden};
use crate::AppState;
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use serde::{Deserialize, Serialize};
use std::{future::Future, pin::Pin};
use utoipa::ToSchema;

pub enum Authorization {
    User { id: i64, token: String },
    Site { id: i64, token: String },
}

fn tokenizer<const N: usize>(value: Option<&str>) -> Result<[&str; N], AppErr> {
    if value.is_none() {
        return Err(AppErrForbidden("invalid authorization value"));
    }
    let result: [&str; N] = value
        .unwrap()
        .splitn(N, ':')
        .collect::<Vec<&str>>()
        .try_into()
        .map_err(|_| AppErrForbidden("invalid authorization token"))?;

    Ok(result)
}

impl TryFrom<&str> for Authorization {
    type Error = AppErr;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut tokens = value.splitn(2, ' ');
        let key = tokens.next().and_then(|v| Some(v.to_lowercase()));
        if key.is_none() {
            return Err(AppErrForbidden("auth key was not found"));
        }

        match key.unwrap().as_str() {
            "user" => {
                let [id, token] = tokenizer(tokens.next())?;
                Ok(Authorization::User {
                    id: id.parse()?,
                    token: token.to_string(),
                })
            }
            "site" => {
                let [id, token] = tokenizer(tokens.next())?;
                Ok(Authorization::Site {
                    id: id.parse()?,
                    token: token.to_string(),
                })
            }
            key => Err(AppErrForbidden(&format!("unknown key in auth: {key}"))),
        }
    }
}

impl TryFrom<&HttpRequest> for Authorization {
    type Error = AppErr;

    fn try_from(rq: &HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = rq.headers().get("authorization") {
            return Authorization::try_from(value.to_str()?);
        }

        for hdr in rq.headers().get_all("cookie") {
            for cookie in hdr.as_bytes().split(|v| *v == b';') {
                let mut s = cookie.splitn(2, |v| *v == b'=');

                let k = s.next().and_then(|v| String::from_utf8(v.into()).ok());
                let v = s.next().and_then(|v| String::from_utf8(v.into()).ok());
                if k.is_none() || v.is_none() {
                    continue;
                }

                if k.unwrap().trim().to_lowercase() == "authorization" {
                    return Authorization::try_from(v.unwrap().as_str());
                }
            }
        }

        Err(AppErrForbidden("no authorization"))
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, ToSchema, Default)]
pub struct User {
    pub id: i64,
    pub phone: String,
    pub name: String,
    pub token: Option<String>,
    pub admin: bool,
}

impl FromRequest for User {
    type Error = AppErr;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _pl: &mut Payload) -> Self::Future {
        let state = req.app_data::<Data<AppState>>().unwrap();
        let pool = state.sql.clone();
        let auth = Authorization::try_from(req);
        // let token = BearerAuth::from_request(req, pl);

        Box::pin(async move {
            let user = match auth? {
                Authorization::User { id, token } => {
                    sqlx::query_as! {
                        User,
                        "select * from users where id = ? and token = ?",
                        id, token
                    }
                    .fetch_one(&pool)
                    .await?
                }
                _ => return Err(AppErrForbidden("invalid auth")),
            };

            Ok(user)
        })
    }
}

pub struct Admin(pub User);
inner_deref!(Admin, User);

impl FromRequest for Admin {
    type Error = AppErr;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let user = User::from_request(req, payload);
        Box::pin(async {
            let user = user.await?;
            if !user.admin {
                return Err(AppErrForbidden("forbidden"));
            }

            Ok(Admin(user))
        })
    }
}
