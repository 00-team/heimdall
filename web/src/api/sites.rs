use actix_web::web::{self, Data, Json, Query};
use actix_web::{get, post, rt, HttpRequest, HttpResponse, Scope};
use actix_ws::AggregatedMessage;
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use utoipa::{OpenApi, ToSchema};

use crate::docs::UpdatePaths;
use crate::models::site::{SiteMessage, Status};
use crate::models::user::{Authorization, User};
use crate::models::{site::Site, Response};
use crate::models::{AppErr, AppErrNotFound, ListInput};
use crate::utils::CutOff;
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::sites")),
    paths(list, dump, ping, live, message_add, message_list),
    components(schemas(
        Site, Status, SiteDumpBody, SiteMessage, SiteAddMessageBody
    )),
    servers((url = "/sites")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    params(ListInput),
    responses((status = 200, body = Vec<Site>))
)]
/// List
#[get("/")]
async fn list(
    _: User, q: Query<ListInput>, state: Data<AppState>,
) -> Response<Vec<Site>> {
    let offset = q.page * 32;
    let sites = sqlx::query_as! {
        Site,
        "select * from sites limit 32 offset ?",
        offset
    }
    .fetch_all(&state.sql)
    .await?;

    Ok(Json(sites))
}

#[derive(Deserialize, ToSchema)]
struct SiteDumpBody {
    total: i64,
    total_time: i64,
    max_time: i64,
    min_time: i64,
    status: HashMap<String, Status>,
}

#[utoipa::path(
    post,
    request_body = SiteDumpBody,
    responses((status = 200))
)]
/// Dump
#[post("/dump/")]
async fn dump(
    rq: HttpRequest, body: Json<SiteDumpBody>, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let mut sites = state.sites.lock().await;
    let site = match Authorization::try_from(&rq)? {
        Authorization::Site { id, token } => sites
            .get_mut(&id)
            .and_then(|v| if v.token == Some(token) { Some(v) } else { None })
            .and_then(|v| if v.online { Some(v) } else { None })
            .ok_or(()),
        _ => Err(()),
    }
    .map_err(|_| AppErrNotFound("no site was found"))?;

    site.total_requests += body.total;
    site.total_requests_time += body.total_time;
    site.requests_max_time = body.max_time.max(site.requests_max_time);
    site.requests_min_time = body.min_time.min(site.requests_min_time);
    site.latest_request = utils::now();
    for (sk, ns) in body.status.iter() {
        if let Some(os) = site.status.get_mut(sk) {
            os.count += ns.count;
            os.total_time += ns.total_time;
            os.max_time = os.max_time.max(ns.max_time);
            os.min_time = os.min_time.min(ns.min_time);
        } else {
            site.status.insert(sk.clone(), ns.clone());
        }
    }

    sqlx::query! {"
        update sites set
        total_requests = ?,
        total_requests_time = ?,
        requests_max_time = ?,
        requests_min_time = ?,
        latest_request = ?,
        status = ?
        where id = ?
    ",
        site.total_requests,
        site.total_requests_time,
        site.requests_max_time,
        site.requests_min_time,
        site.latest_request,
        site.status,
        site.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    post,
    responses((status = 200))
)]
/// Ping
#[post("/ping/")]
async fn ping(
    rq: HttpRequest, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let mut sites = state.sites.lock().await;
    let site = match Authorization::try_from(&rq)? {
        Authorization::Site { id, token } => sites
            .get_mut(&id)
            .and_then(|v| if v.token == Some(token) { Some(v) } else { None })
            .and_then(|v| if v.online { Some(v) } else { None })
            .ok_or(()),
        _ => Err(()),
    }
    .map_err(|_| AppErrNotFound("no site was found"))?;

    site.latest_ping = utils::now();

    sqlx::query! {
        "update sites set latest_ping = ? where id = ?",
        site.latest_ping, site.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, ToSchema)]
struct SiteAddMessageBody {
    text: String,
    tag: String,
}

#[utoipa::path(
    post,
    request_body = SiteAddMessageBody,
    responses((status = 200, body = SiteMessage))
)]
/// Message Add
#[post("/messages/")]
async fn message_add(
    rq: HttpRequest, body: Json<SiteAddMessageBody>, state: Data<AppState>,
) -> Response<SiteMessage> {
    let mut sites = state.sites.lock().await;
    let site = match Authorization::try_from(&rq)? {
        Authorization::Site { id, token } => sites
            .get_mut(&id)
            .and_then(|v| if v.token == Some(token) { Some(v) } else { None })
            .ok_or(()),
        _ => Err(()),
    }
    .map_err(|_| AppErrNotFound("no site was found"))?;

    let mut text = body.text.clone();
    let mut tag = body.tag.clone();
    text.cut_off(2048);
    tag.cut_off(255);

    let timestamp = utils::now();
    let mut msg = SiteMessage { id: 0, timestamp, text, tag, site: site.id };
    site.latest_message_timestamp = msg.timestamp;

    let result = sqlx::query! {
        "insert into sites_messages(site, timestamp, text, tag) values(?,?,?,?)",
        msg.site, msg.timestamp, msg.text, msg.tag
    }
    .execute(&state.sql)
    .await?;

    sqlx::query!{
        "delete from sites_messages where site = ? AND id < (select id from sites_messages where site = ? order by id desc limit 1 offset 32)",
        msg.site, msg.site
    }
    .execute(&state.sql)
    .await?;

    msg.id = result.last_insert_rowid();

    Ok(Json(msg))
}

#[utoipa::path(
    get,
    params(("site_id" = i64, Path, example = 1)),
    responses((status = 200, body = Vec<SiteMessage>))
)]
/// Message List
#[get("/{site_id}/messages/")]
async fn message_list(
    _: User, site: Site, state: Data<AppState>,
) -> Response<Vec<SiteMessage>> {
    let messages = sqlx::query_as! {
        SiteMessage,
        "select * from sites_messages where site = ? order by id desc",
        site.id
    }
    .fetch_all(&state.sql)
    .await?;

    Ok(Json(messages))
}

#[utoipa::path(get)]
/// live
#[get("/live/")]
async fn live(
    rq: HttpRequest, stream: web::Payload, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let (res, mut session, stream) = actix_ws::handle(&rq, stream)?;

    let mut stream = stream
        .aggregate_continuations()
        // aggregate continuation frames up to 1MiB
        .max_continuation_size(1024 * 1024);

    // let _ = session.text("welcome kid").await;

    rt::spawn(async move {
        while let Some(msg) = stream.next().await {
            if let Err(e) = &msg {
                log::info!("ws msg err: {e:?}");
            }

            // log::info!("ws msg: {msg:?}");
            match msg.unwrap() {
                AggregatedMessage::Text(txt) => {
                    let id = txt.parse::<i64>();
                    if id.is_err() {
                        return;
                    }
                    let sites = state.sites.lock().await;
                    let site = sites.get(&id.unwrap());
                    if site.is_none() {
                        let _ = session
                            .text(
                                serde_json::to_string(&AppErrNotFound(
                                    "site not found",
                                ))
                                .unwrap(),
                            )
                            .await;
                        return;
                    }
                    let _ = session
                        .text(serde_json::to_string(site.unwrap()).unwrap())
                        .await;
                }
                AggregatedMessage::Ping(bytes) => {
                    let _ = session.pong(&bytes).await;
                }
                _ => {}
            }
        }
    });

    Ok(res)
}

pub fn router() -> Scope {
    Scope::new("/sites")
        .service(list)
        .service(dump)
        .service(ping)
        .service(message_add)
        .service(message_list)
        .service(live)
}
