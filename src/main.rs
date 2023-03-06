use anyhow::Result;
use axum::{extract::Path, handler::get, http::StatusCode, Router};
use percent_encoding::percent_decode_str;
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    convert::TryInto,
    hash::{ Hash, Hasher},
    sync::Arc
};
use axum::extract::Extension;
use axum::http::{HeaderMap, HeaderValue};
use bytes::Bytes;

use tokio::sync::Mutex;
use tower::ServiceBuilder;
use tracing::{info, instrument};

use lru::LruCache;
use tower_http::add_extension::AddExtensionLayer;
use crate::pb::ImageSpec;


mod pb;

#[derive(Deserialize)]
struct Params {
    spec: String,
    url: String
}

type Cache = Arc<Mutex<LruCache<u64, Bytes>>>;

#[instrument(level = "info", skip(cache))]
async fn retrieve_image(url: &str, cache: Cache) -> Result<Bytes> {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let key = hasher.finish();
    let g = &mut cache.lock().await;
    let data = match g.get(&key) {
        Some(v) => {
            info!("match cache {}", key);
            v.to_owned()
        }
        None => {
            info!("retrieve url");
            let resp = reqwest::get(url).await?;
            let data = resp.bytes().await?;
            g.put(key, data.clone());
            data
        }
    };
    Ok(data)

}

async fn generate(
    Path(Params { spec, url}): Path<Params>,
    Extension(cache): Extension<Cache>,
) -> Result<(HeaderMap, Vec<u8>), StatusCode>{
    let spec: ImageSpec = spec
        .as_str()
        .try_into()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let url: &str = &percent_decode_str(&url).decode_utf8_lossy();
    let data = retrieve_image(&url, cache)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let mut headers = HeaderMap::new();
    headers.insert("Content-type", HeaderValue::from_static("image/jepg"));
    Ok(( headers, data.to_vec()))
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    tracing_subscriber::fmt::init();

    let cache: Cache = Arc::new(Mutex::new(LruCache::new(1024)));

    let app = Router::new()
        .route("/image/:spec/:url", get(generate))
        .layer(
            ServiceBuilder::new()
                .layer(AddExtensionLayer::new(cache))
                .into_inner(),
        );
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("listen on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


