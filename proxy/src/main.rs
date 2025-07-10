use axum::{routing::any, Router, extract::Request, response::Response};
use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::time::{sleep, Duration};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let is_up = Arc::new(AtomicBool::new(true));
    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(is_up.clone());

    tokio::spawn(health_monitor(is_up.clone()));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await.unwrap();
}

async fn handler(
    state: axum::extract::State<Arc<AtomicBool>>,
    req: Request,
) -> Response {
    let client = Client::new();
    let primary = "http://localhost:4000";
    let fallback = "http://localhost:5000";
    let url = if state.load(Ordering::SeqCst) {
        primary
    } else {
        fallback
    };

    let path = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("/");
    let uri = format!("{}{}", url, path);

    let forwarded = client.request(req.method().clone(), &uri)
        .body(hyper::body::to_bytes(req.into_body()).await.unwrap())
        .send()
        .await;

    match forwarded {
        Ok(resp) => {
            let mut response = Response::builder().status(resp.status());
            let body = resp.bytes().await.unwrap_or_default();
            response.body(axum::body::Body::from(body)).unwrap()
        }
        Err(_) => {
            // Switch to fallback if not already
            state.store(false, Ordering::SeqCst);
            Response::builder()
                .status(502)
                .body("Primary failed, using fallback.".into())
                .unwrap()
        }
    }
}

async fn health_monitor(is_up: Arc<AtomicBool>) {
    let client = Client::new();
    let health_url = "http://localhost:4000/health";

    loop {
        if !is_up.load(Ordering::SeqCst) {
            if let Ok(resp) = client.get(health_url).send().await {
                if resp.status().is_success() {
                    is_up.store(true, Ordering::SeqCst);
                    println!("Primary is healthy again");
                }
            }
        }
        sleep(Duration::from_secs(5)).await;
    }
}
