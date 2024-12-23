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
    paths(add, update, reset, del),
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

    let now = utils::now();
    let result = sqlx::query! {
        "insert into sites(name, timestamp) values(?, ?)",
        body.name, now
    }
    .execute(&state.sql)
    .await?;

    let site = Site {
        id: result.last_insert_rowid(),
        name: body.name.clone(),
        timestamp: now,
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
    online: bool,
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

    site.online = body.online;
    site.name = body.name.clone();
    if body.token {
        site.token = Some(utils::get_random_string(Config::SITE_TOKEN_ABC, 41));
    }

    sqlx::query! {
        "update sites set name = ?, token = ?, online = ? where id = ?",
        site.name, site.token, site.online, site.id
    }
    .execute(&state.sql)
    .await?;

    let mut sites = state.sites.lock().await;
    let state_site = sites.get_mut(&site.id).expect("unreachable");
    state_site.name = site.name.clone();
    state_site.token = site.token.clone();
    state_site.online = site.online;

    Ok(Json(site))
}

#[utoipa::path(
    patch,
    params(("site_id" = i64, Path, example = 1)),
    responses((status = 200, body = Site))
)]
/// Reset
#[patch("/{site_id}/reset/")]
async fn reset(_: Admin, site: Site, state: Data<AppState>) -> Response<Site> {
    let mut site = site;

    site.total_requests = 0;
    site.total_requests_time = 0;
    site.requests_max_time = 0;
    site.requests_min_time = 0;
    site.status.0.clear();
    site.timestamp = utils::now();

    sqlx::query! {
        r##"update sites set total_requests = 0, total_requests_time = 0,
        requests_max_time = 0, requests_min_time = 0, status = "{}",
        timestamp = ? where id = ?"##,
        site.timestamp, site.id
    }
    .execute(&state.sql)
    .await?;

    let mut sites = state.sites.lock().await;
    let state_site = sites.get_mut(&site.id).expect("unreachable");
    state_site.clone_from(&site);

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
    Scope::new("/sites")
        .service(add)
        .service(update)
        .service(reset)
        .service(del)
}
