use std::path::PathBuf;

use actix_web::web::{Data, Json, Path, Query};
use actix_web::{get, post, FromRequest, HttpRequest, HttpResponse, Scope};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::{OpenApi, ToSchema};

use crate::config::config;
use crate::docs::UpdatePaths;
use crate::models::deploy::{Deploy, DeployStatus};
use crate::models::user::User;
use crate::models::{bad_request, not_found, AppErr, ListInput};
use crate::models::{forbidden, Response};
use crate::{utils, AppState};

#[derive(OpenApi)]
#[openapi(
    tags((name = "api::deploy")),
    paths(list, add),
    components(schemas(
        Deploy, DeployStatus
    )),
    servers((url = "/deploy")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    params(ListInput),
    responses((status = 200, body = Vec<Deploy>))
)]
/// List
#[get("/")]
async fn list(
    _: User, q: Query<ListInput>, state: Data<AppState>,
) -> Response<Vec<Deploy>> {
    let offset = q.page * 32;
    let deploy = sqlx::query_as! {
        Deploy,
        "select * from deploys order by id desc limit 32 offset ?",
        offset
    }
    .fetch_all(&state.sql)
    .await?;

    Ok(Json(deploy))
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEventAuthor {
    name: String,
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEventCommit {
    message: String,
    author: GithubPushEventAuthor,
}

#[derive(Deserialize, ToSchema)]
struct GithubPushEvent {
    commits: Vec<GithubPushEventCommit>,
}

async fn do_deploy(pool: SqlitePool, repo: String, path: PathBuf, id: i64) {
    let now = utils::now();
    let _ = sqlx::query! {
        "update deploys set begin = ?, status = ? where id = ?",
        now, DeployStatus::Running, id
    }
    .execute(&pool)
    .await;

    let res = tokio::process::Command::new(path).output().await;
    let now = utils::now();

    let (status, stdout, stderr) = match res {
        Ok(v) => (
            // v.status;
            DeployStatus::Success,
            Some(String::from_utf8(v.stdout).unwrap_or_default()),
            Some(String::from_utf8(v.stderr).unwrap_or_default()),
        ),
        Err(e) => (DeployStatus::Failed, None, Some(e.to_string())),
    };

    let _ = sqlx::query! {
        "update deploys set finish = ?, stdout = ?, stderr = ?, status = ? where id = ?",
        now, stdout, stderr, status, id
    }
    .execute(&pool)
    .await;

    let pending = sqlx::query_as! {
        Deploy,
        "select * from deploys where status = ? AND repo = ? limit 1",
        DeployStatus::Pending, repo
    }
    .fetch_optional(&pool)
    .await;

    let Ok(Some(pending)) = pending else { return };
    if let Some(path) = config().deploy_repo.get(&pending.repo) {
        Box::pin(do_deploy(pool, repo, path.clone(), pending.id)).await;
    }
}

#[utoipa::path(
    post,
    params(
        ("repo" = String, Path, example = "simurgh"),
        ("actor" = String, Path, example = "007"),
        ("pass" = String, Path, example = "<password>"),
    ),
    responses((status = 200))
)]
/// Add
#[post("/{repo}/{actor}/{pass}/")]
async fn add(
    rq: HttpRequest, path: Path<(String, String, String)>,
    state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let (repo, actor, pass) = path.into_inner();

    let config = config();
    let path = config.deploy_repo.get(&repo);

    let Some(path) = path else {
        return Err(not_found!("repo not found"));
    };

    if config.deploy_key != pass {
        return Err(forbidden!("invalid password"));
    }

    let deploys = sqlx::query_as! {
        Deploy,
        "select * from deploys where repo = ? and finish = 0",
        repo,
    }
    .fetch_all(&state.sql)
    .await?;

    let running = deploys.iter().find(|d| d.status == DeployStatus::Running);
    let pending =
        deploys.iter().cloned().find(|d| d.status == DeployStatus::Pending);

    if pending.is_some() && running.is_some() {
        return Ok(HttpResponse::Ok().finish());
    }

    let sender = match actor.as_str() {
        "github" => {
            if !matches!(
                rq.headers().get("X-GitHub-Event").map(|v| v.to_str()),
                Some(Ok("push"))
            ) {
                return Err(bad_request!("bad github event. only push"));
            }

            let Ok(event) = Json::<GithubPushEvent>::extract(&rq).await else {
                return Err(bad_request!("invalid body"));
            };

            let Some(commit) = event
                .commits
                .iter()
                .rev()
                .find(|c| c.message.contains("release"))
            else {
                return Ok(HttpResponse::Ok().finish());
            };

            commit.author.name.clone()
        }
        "sadra" | "007" => actor.clone(),
        _ => return Err(forbidden!("unknown actor")),
    };

    let id = sqlx::query! {
        "insert into deploys(repo, actor, sender, status) values(?,?,?,?)",
        repo, actor, sender, DeployStatus::Pending
    }
    .execute(&state.sql)
    .await?
    .last_insert_rowid();

    if running.is_some() {
        return Ok(HttpResponse::Ok().finish());
    }

    if let Some(d) = pending {
        if let Some(path) = config.deploy_repo.get(&d.repo) {
            tokio::spawn(do_deploy(
                state.sql.clone(),
                d.repo,
                path.clone(),
                d.id,
            ));
            return Ok(HttpResponse::Ok().finish());
        }

        sqlx::query! {
            "update deploys set status = ?, stderr = 'repo not found' where id = ?",
            DeployStatus::Failed, d.id
        }
        .execute(&state.sql)
        .await?;
    }

    tokio::spawn(do_deploy(state.sql.clone(), repo, path.clone(), id));

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/deploy").service(list).service(add)
}
