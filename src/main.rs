use axum::{
    body::Body,
    extract::{Request, State},
    http::HeaderValue,
    http::{header, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Router,
};

use once_cell::sync::Lazy;
use regex::Regex;
use std::net::SocketAddr;
use sync_wrapper::SyncStream;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type Client = reqwest::Client;

static EXP1: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:releases|archive)\/.*$").unwrap()
});

static EXP2: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:blob|raw)\/.*$").unwrap());

static EXP3: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/(?:info|git-).*").unwrap());

static EXP4: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?raw\.(?:githubusercontent|github)\.com\/.+?\/.+?\/.+?\/.+$")
        .unwrap()
});

static EXP5: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:https?:\/\/)?gist\.(?:githubusercontent|github)\.com\/.+?\/.+?\/.+$").unwrap()
});

static EXP6: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:https?:\/\/)?github\.com\/.+?\/.+?\/tags.*$").unwrap());

static CONFIG_JSDELIVR: bool = false;

fn handle_204() -> Result<Response, StatusCode> {
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::NO_CONTENT;
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET,POST,PUT,PATCH,TRACE,DELETE,HEAD,OPTIONS"),
    );
    res.headers_mut().insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("1728000"),
    );
    return Ok(res);
}

fn handle_redirect(query_string: String) -> Result<Response, StatusCode> {
    let location = format!("/{}", query_string);
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::FOUND;
    res.headers_mut().insert(
        header::LOCATION,
        HeaderValue::from_str(location.as_str()).unwrap(),
    );
    return Ok(res);
}

fn get_git_url(req: &Request<Body>) -> String {
    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(req.uri().path());

    let path = if path_query.starts_with("/") {
        path_query[1..].into()
    } else {
        path_query
    };
    path.into()
}

async fn handle_proxy(
    mut req: Request<Body>,
    client: Client,
    path_query: String,
) -> Result<Response, StatusCode> {
    req.headers_mut().remove(header::HOST);

    *req.uri_mut() = Uri::try_from(path_query).unwrap();
    let req = req.map(|body| reqwest::Body::wrap_stream(SyncStream::new(body.into_data_stream())));
    let req = reqwest::Request::try_from(req).unwrap();
    let res = client.execute(req).await.unwrap();

    let mut builder = Response::builder().status(res.status());
    *builder.headers_mut().unwrap() = res.headers().clone();
    Ok(builder
        .body(axum::body::Body::from_stream(res.bytes_stream()))
        .unwrap())
}

async fn handler(State(client): State<Client>, req: Request<Body>) -> Result<Response, StatusCode> {
    //  option
    if req.method() == Method::OPTIONS
        && req
            .headers()
            .contains_key(header::ACCESS_CONTROL_REQUEST_HEADERS)
    {
        return handle_204();
    }
    let path = get_git_url(&req);

    // redirect
    if path.starts_with("q=") {
        return handle_redirect(path.replace("q=", ""));
    }

    if EXP1.is_match(&path)
        || EXP5.is_match(&path)
        || EXP6.is_match(&path)
        || EXP3.is_match(&path)
        || EXP4.is_match(&path)
    {
        return handle_proxy(req, client, path).await;
    }

    if EXP2.is_match(&path) {
        if CONFIG_JSDELIVR {
            let new_url = path.replacen("/blob/", "@", 1).replacen(
                "github.com",
                "https://gcore.jsdelivr.net/gh",
                1,
            );
            return handle_redirect(new_url);
        } else {
            let path = path.replacen("/blob/", "/raw/", 1);
            return handle_proxy(req, client, path).await;
        }
    }

    Ok("Proxy response placeholder".into_response())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example-basic=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let client = reqwest::Client::builder().build().unwrap();

    let app = Router::new().fallback(handler).with_state(client);
    let addr = SocketAddr::from(([127, 0, 0, 1], 4000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
