use std::collections::HashMap;

use actix_multipart::form::tempfile::TempFile;
use actix_multipart::form::MultipartForm;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, post, put, HttpResponse, Scope};
use serde::Deserialize;
use sqlx::SqlitePool;
use utoipa::{IntoParams, OpenApi, ToSchema};

use crate::docs::UpdatePaths;
use crate::models::product::{Product, ProductKind, ProductPart, ProductTag};
use crate::models::user::Admin;
use crate::models::{AppErr, AppErrBadRequest, JsonStr, Response};
use crate::utils::{self, CutOff};
use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    tags((name = "admin::product")),
    paths(
        product_list, product_get, product_add, product_update, product_delete,
        product_thumbnail_update, product_thumbnail_delete,
        product_photo_add, product_photo_del
    ),
    components(schemas(
        Product, ProductTag, ProductKind, ProductPart, ProductAddBody,
        ProductUpdateBody
    )),
    servers((url = "/products")),
    modifiers(&UpdatePaths)
)]
pub struct ApiDoc;

#[derive(Deserialize, IntoParams)]
pub struct ProductListBody {
    #[param(example = 0)]
    pub page: u32,
    pub best: Option<bool>,
}

#[utoipa::path(
    get,
    params(ProductListBody),
    responses((status = 200, body = Vec<Product>))
)]
/// Product List
#[get("/")]
async fn product_list(
    _: Admin, q: Query<ProductListBody>, state: Data<AppState>,
) -> Response<Vec<Product>> {
    let offset = i64::from(q.page) * 32;

    let products = if let Some(best) = q.best {
        sqlx::query_as! {
            Product,
            "select * from products where best = ?
            order by id desc limit 32 offset ?",
            best, offset
        }
        .fetch_all(&state.sql)
        .await?
    } else {
        sqlx::query_as! {
            Product,
            "select * from products order by id desc limit 32 offset ?",
            offset
        }
        .fetch_all(&state.sql)
        .await?
    };

    Ok(Json(products))
}

#[utoipa::path(
    get,
    params(("id" = i64, Path,)),
    responses((status = 200, body = Product))
)]
/// Product Get
#[get("/{id}/")]
async fn product_get(_: Admin, product: Product) -> Json<Product> {
    Json(product)
}

#[derive(Deserialize, ToSchema)]
struct ProductAddBody {
    name: String,
    code: String,
    kind: ProductKind,
    slug: String,
}

#[utoipa::path(
    post,
    request_body = ProductAddBody,
    responses((status = 200, body = Product))
)]
/// Add
#[post("/")]
async fn product_add(
    _: Admin, body: Json<ProductAddBody>, state: Data<AppState>,
) -> Response<Product> {
    let now = utils::now();
    let mut body = body;

    utils::verify_slug(&body.slug)?;

    body.name.cut_off(255);
    body.code.cut_off(255);
    body.slug.cut_off(255);

    let result = sqlx::query! {
        "insert into products(slug, kind, name, code, created_at)
        values(?,?,?,?,?)",
        body.slug, body.kind, body.name, body.code, now
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(Product {
        id: result.last_insert_rowid(),
        slug: body.slug.clone(),
        kind: body.kind.clone(),
        name: body.name.clone(),
        code: body.code.clone(),
        created_at: now,
        ..Default::default()
    }))
}

async fn update_tag(
    old: Option<i64>, new: Option<i64>, kind: &ProductKind, part: ProductPart,
    pool: &SqlitePool,
) -> Result<Option<i64>, AppErr> {
    if old.is_none() && new.is_none() {
        return Ok(None);
    }

    if matches!((old, new), (Some(a), Some(b)) if a == b) {
        log::info!("same tag: {old:?} == {new:?}");
        return Ok(new);
    }

    if let Some(id) = old {
        sqlx::query! {
            "update product_tags set count = count - 1 where id = ?", id
        }
        .execute(pool)
        .await?;
    }

    if let Some(id) = new {
        let tag = sqlx::query_as! {
            ProductTag,
            "select * from product_tags where id = ? AND kind = ? AND part = ?",
            id, kind, part
        }
        .fetch_one(pool)
        .await?;

        sqlx::query! {
            "update product_tags set count = count + 1 where id = ?", tag.id
        }
        .execute(pool)
        .await?;

        return Ok(Some(tag.id));
    }

    return Ok(None);
}

#[derive(Deserialize, ToSchema)]
struct ProductUpdateBody {
    name: String,
    slug: String,
    code: String,
    detail: String,
    description: String,
    specification: HashMap<String, String>,
    tag_leg: Option<i64>,
    tag_bed: Option<i64>,
    best: bool,
}

#[utoipa::path(
    patch,
    params(("id" = i64, Path,)),
    request_body = ProductUpdateBody,
    responses((status = 200, body = Product))
)]
/// Update
#[patch("/{id}/")]
async fn product_update(
    _: Admin, product: Product, body: Json<ProductUpdateBody>,
    state: Data<AppState>,
) -> Response<Product> {
    let mut product = product;

    utils::verify_slug(&body.slug)?;

    product.slug = body.slug.clone();
    product.name = body.name.clone();
    product.code = body.code.clone();
    product.detail = body.detail.clone();
    product.description = body.description.clone();
    product.specification = JsonStr(body.specification.clone());
    product.best = body.best;
    product.updated_at = utils::now();

    product.slug.cut_off(256);
    product.name.cut_off(256);
    product.code.cut_off(256);
    product.detail.cut_off(2048);
    product.description.cut_off(512);

    product.tag_leg = update_tag(
        product.tag_leg,
        body.tag_leg,
        &product.kind,
        ProductPart::Leg,
        &state.sql,
    )
    .await?;

    product.tag_bed = update_tag(
        product.tag_bed,
        body.tag_bed,
        &product.kind,
        ProductPart::Bed,
        &state.sql,
    )
    .await?;

    sqlx::query! {
        "update products set slug = ?, name = ?, code = ?, detail = ?, best = ?,
        updated_at = ?, tag_leg = ?, tag_bed = ?, description = ?,
        specification = ? where id = ?",
        product.slug, product.name, product.code, product.detail, product.best,
        product.updated_at, product.tag_leg, product.tag_bed, product.description,
        product.specification, product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(product))
}

#[utoipa::path(
    delete,
    params(("id" = i64, Path,)),
    responses((status = 200))
)]
/// Product Delete
#[delete("/{id}/")]
async fn product_delete(
    _: Admin, product: Product, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    if let Some(t) = product.thumbnail {
        utils::remove_record(&format!("pt-{}-{t}", product.id));
    }

    for p in product.photos.0 {
        utils::remove_record(&format!("pp-{}-{p}", product.id));
    }

    sqlx::query! {
        "delete from products where id = ?",
        product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, MultipartForm, ToSchema)]
pub struct ProductPhoto {
    #[schema(value_type = String, format = Binary)]
    #[multipart(limit = "20MB")]
    pub photo: TempFile,
}

#[utoipa::path(
    put,
    params(("id" = i64, Path,)),
    request_body(content = ProductPhoto, content_type = "multipart/form-data"),
    responses((status = 200, body = Product))
)]
/// Set Thumbnail
#[put("/{id}/thumbnail/")]
async fn product_thumbnail_update(
    _: Admin, product: Product, form: MultipartForm<ProductPhoto>,
    state: Data<AppState>,
) -> Response<Product> {
    let mut product = product;
    let salt = if let Some(s) = &product.thumbnail {
        s.clone()
    } else {
        let s = utils::get_random_bytes(8);
        product.thumbnail = Some(s.clone());
        s
    };

    let filename = format!("pt-{}-{}", product.id, salt);
    utils::save_photo(form.photo.file.path(), &filename, (2200, 1000))?;

    sqlx::query_as! {
        Product,
        "update products set thumbnail = ? where id = ?",
        product.thumbnail, product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(product))
}

#[utoipa::path(
    delete,
    params(("id" = i64, Path,)),
    responses((status = 200, body = Product))
)]
/// Delete Thumbnail
#[delete("/{id}/thumbnail/")]
async fn product_thumbnail_delete(
    _: Admin, product: Product, state: Data<AppState>,
) -> Response<Product> {
    let mut product = product;

    if let Some(t) = product.thumbnail {
        utils::remove_record(&format!("pt-{}-{t}", product.id));
        product.thumbnail = None;
    }

    sqlx::query_as! {
        Product,
        "update products set thumbnail = ? where id = ?",
        product.thumbnail, product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(product))
}

#[utoipa::path(
    put,
    params(("id" = i64, Path,)),
    request_body(content = ProductPhoto, content_type = "multipart/form-data"),
    responses((status = 200, body = Product))
)]
/// Add Photo
#[put("/{id}/photos/")]
async fn product_photo_add(
    _: Admin, product: Product, form: MultipartForm<ProductPhoto>,
    state: Data<AppState>,
) -> Response<Product> {
    if product.photos.len() >= 255 {
        return Err(AppErrBadRequest("too many photos"));
    }

    let mut product = product;
    let salt = loop {
        let s = utils::get_random_bytes(8);
        if !product.photos.iter().any(|p| p == &s) {
            break s;
        }
    };
    product.photos.push(salt.clone());

    let filename = format!("pp-{}-{}", product.id, salt);
    utils::save_photo(form.photo.file.path(), &filename, (1024, 1024))?;

    sqlx::query_as! {
        Product,
        "update products set photos = ? where id = ?",
        product.photos, product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(Json(product))
}

#[utoipa::path(
    delete,
    params(("id" = i64, Path,), ("idx" = u8, Path,)),
    responses((status = 200))
)]
/// Delete Photo
#[delete("/{id}/photos/{idx}/")]
async fn product_photo_del(
    _: Admin, product: Product, path: Path<(i64, u8)>, state: Data<AppState>,
) -> Result<HttpResponse, AppErr> {
    let mut product = product;
    let idx: usize = path.1.into();
    if idx >= product.photos.len() {
        return Err(AppErrBadRequest("photo not found"));
    }

    let salt = product.photos.remove(idx);
    utils::remove_record(&format!("pp-{}-{}", product.id, salt));

    sqlx::query_as! {
        Product,
        "update products set photos = ? where id = ?",
        product.photos, product.id
    }
    .execute(&state.sql)
    .await?;

    Ok(HttpResponse::Ok().finish())
}

pub fn router() -> Scope {
    Scope::new("/products")
        .service(product_list)
        .service(product_get)
        .service(product_add)
        .service(product_update)
        .service(product_delete)
        .service(product_thumbnail_update)
        .service(product_thumbnail_delete)
        .service(product_photo_add)
        .service(product_photo_del)
}
