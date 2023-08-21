use axum::{
    extract::{Multipart, Path},
    routing::get,
    routing::post,
    Json, Router,
};
use hyper::{Body, Response, StatusCode};
// use multipart::server::Multipart;
use std::{
    net::SocketAddr,
    path::{self},
};
use tokio::fs;
// use tokio_stream::wrappers::BytesStream;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

async fn serve_image(Path(filename): Path<String>) -> Result<Response<Body>, String> {
    if filename.find("..").is_some() || filename.find("/").is_some() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into());
    }
    if filename.find(".png").is_none() {
        return Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into());
    }
    let path = format!("uploads/{}", filename);
    if let Ok(data) = fs::read(&path).await {
        Ok(hyper::Response::new(Body::from(data)))
    } else {
        Ok(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not Found"))
            .unwrap()
            .into())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct UploadResp {
    dst: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct UploadReq {
    file: String,
}

async fn upload(data: String) -> Result<Json<UploadResp>, StatusCode> {
    let data: UploadReq = serde_json::from_str(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
    let data = base64::decode(data.file).map_err(|_| StatusCode::BAD_REQUEST)?;
    let base_path = std::path::Path::new("uploads");
    if !base_path.exists() {
        fs::create_dir(base_path).await.unwrap();
    }
    let file_name = format!("{}.png", Uuid::new_v4());
    // let filename = format!("uploads/1.png");
    fs::write(&base_path.join(&file_name), data).await.unwrap();

    Ok(Json(UploadResp {
        dst: format!(
            "https://img.coderluny.com:444/{}",
            base_path.join(&file_name).to_string_lossy()
        ),
    }))
}

#[tokio::main]
async fn main() {
    // std::env::set_var("RUST_LOG", "trace");
    env_logger::init();
    let app = Router::new()
        .route("/upload", post(upload))
        .route("/uploads/:filename", get(serve_image));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

