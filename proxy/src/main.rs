mod config;


use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use std::str::FromStr;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Error, http};
use futures::StreamExt;
use bytes::{BytesMut};
use tokio::time::sleep;
use reqwest::header::HeaderName;
use tracing::{info};
use tracing_subscriber::fmt;
use crate::config::Settings;

#[derive(Clone)]
struct AppState {
    client: reqwest::Client,
    url_default: String,
    url_fallback: String,
    circuit_open: Arc<AtomicBool>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_tracing();

    let settings = Settings::new();

    let state = AppState {
        client: reqwest::Client::new(),
        url_default: settings.payment_processor_default.clone(),
        url_fallback: settings.payment_processor_fallback.clone(),
        circuit_open: Arc::new(AtomicBool::new(false)),
    };

    // spawn the health-checker
    {
        let hc = state.clone();
        tokio::spawn(async move {
            loop {
                if hc.circuit_open.load(Ordering::SeqCst) {
                    info!("checking default service health");
                    let health_url = format!("{}/payments/service-health", hc.url_default);
                    let client = reqwest::Client::new();

                    if let Ok(resp) = client.get(&health_url).send().await {
                        if resp.status().is_success() {
                            if let Ok(json) = resp.json::<serde_json::Value>().await {
                                if let Some(failing) = json.get("failing").and_then(|v| v.as_bool()) {
                                    if !failing {
                                        info!("default service is healthy again");
                                        hc.circuit_open.store(false, Ordering::SeqCst);
                                    }
                                }
                            }
                        }
                    }
                }

                sleep(Duration::from_secs(5)).await;
            }
        });
    }

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .default_service(web::route().to(proxy_handler))
    })
        .bind((settings.server_url, settings.server_port))?
        .run()
        .await
}

async fn proxy_handler(
    req: HttpRequest,
    mut body: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    info!("proxying request to {}", req.uri());

    // 1) Choose default vs fallback
    let base = if state.circuit_open.load(Ordering::SeqCst) {
        &state.url_fallback
    } else {
        &state.url_default
    };

    // 2) Rebuild target URL (path + query)
    let path_q = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let url = format!("{}{}", base, path_q);

    info!("forwarding request to {}", url);

    // 3) Collect full request body
    let mut buf = BytesMut::new();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(Error::from)?;
        buf.extend_from_slice(&chunk);
    }

    info!("forwarding request with body {}", String::from_utf8_lossy(&buf));

    // 4) Build reqwest request with same method, headers, and body
    let method = reqwest::Method::from_str(req.method().as_str()).unwrap();

    info!("forwarding request with method {}", method);

    let mut builder = state
        .client
        .request(method, &url);

    for (name, value) in req.headers().iter() {
        if let Ok(hdr) = HeaderName::from_str(name.as_str()) {
            builder = builder.header(hdr, value.as_bytes());
        }
    }

    let resp = builder
        .body(buf.freeze())
        .send()
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    info!("received response with status {}", resp.status());

    // 5) Build Actix response from reqwest::Response
    let status = resp.status();
    let mut client_resp = HttpResponse::build(http::StatusCode::from_u16(status.as_u16()).unwrap());
    for (name, value) in resp.headers().iter() {
        let header_name = http::header::HeaderName::from_str(name.as_str()).unwrap();
        let header_value = http::header::HeaderValue::from_str(value.to_str().unwrap())?;
        client_resp.insert_header((header_name, header_value));
    }

    let bytes = resp
        .bytes()
        .await
        .map_err(actix_web::error::ErrorBadGateway)?;

    // Circuit breaker: trip on non-success
    if !status.is_success() {
        state.circuit_open.store(true, Ordering::SeqCst);
    }

    Ok(client_resp.body(bytes))
}

fn init_tracing() {
    fmt()
        .with_line_number(true)
        .init();
}