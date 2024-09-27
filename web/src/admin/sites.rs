use actix_web::web::{Data, Json};
use actix_web::{delete, patch, post, HttpResponse, Scope};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::config::Config;
use crate::docs::UpdatePaths;
use crate::models::user::Admin;
use crate::models::AppErr;
use crate::models::{site::Site, Response};
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin::sites")),
    paths(add, update, del),
    components(schemas(Site, SitesAddBody, SitesUpdateBody)),
    servers((url = "/sites")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Deserialize, ToSchema)]
struct SitesAddBody {
    name: String,
}

#[utoipa::path(
    post,
    request_body = SitesAddBody,
    responses((status = 200, body = Site))
)]
/// Add
#[post("/")]
async fn add(
    _: Admin, body: Json<SitesAddBody>, state: Data<AppState>,
) -> Response<Site> {
    Site::verify_name(&body.name)?;

    let result = sqlx::query! {
        "insert into sites(name) values(?)",
        body.name
    }
    .execute(&state.sql)
    .await?;

    let site = Site {
        id: result.last_insert_rowid(),
        name: body.name.clone(),
        ..Default::default()
    };
    let mut sites = state.sites.lock().await;
    sites.insert(site.id, site.clone());

    Ok(Json(site))
}

#[derive(Deserialize, ToSchema)]
struct SitesUpdateBody {
    name: String,
    token: bool,
}

#[utoipa::path(
    patch,
    params(("site_id" = i64, Path, example = 1)),
    request_body = SitesUpdateBody,
    responses((status = 200, body = Site))
)]
/// Update
#[patch("/{site_id}/")]
async fn update(
    _: Admin, site: Site, body: Json<SitesUpdateBody>, state: Data<AppState>,
) -> Response<Site> {
    let mut site = site;
    Site::verify_name(&body.name)?;

    site.name = body.name.clone();
    if body.token {
        site.token = Some(utils::get_random_string(Config::SITE_TOKEN_ABC, 41));
    }

    sqlx::query! {
        "update sites set name = ?, token = ? where id = ?",
        site.name, site.token, site.id
    }
    .execute(&state.sql)
    .await?;

    let mut sites = state.sites.lock().await;
    let state_site = sites.get_mut(&site.id).expect("unreachable");
    state_site.name = site.name.clone();
    state_site.token = site.token.clone();

    Ok(Json(site))
}

#[utoipa::path(
    delete,
    params(("site_id" = i64, Path, example = 1)),
    responses((status = 200, body = Site))
)]
/// Delete
#[delete("/{site_id}/")]
async fn del(
    _: Admin, site: Site, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    sqlx::query! {
        "delete from sites where id = ?",
        site.id
    }
    .execute(&state.sql)
    .await?;

    let mut sites = state.sites.lock().await;
    sites.remove(&site.id);

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/sites").service(add).service(update).service(del)
}
