use std::net::SocketAddr;

use anyhow::Result;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::sync::{LazyLock, OnceLock};
use tokio::sync::Mutex;

static SERVER_STARTED: OnceLock<()> = OnceLock::new();
static METRICS_GATHER_GUARD: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

const DEFAULT_METRICS_ADDR: &str = "127.0.0.1:59321";

pub fn ensure_metrics_server(app_handle: &tauri::AppHandle) {
    if SERVER_STARTED.get().is_some() {
        return;
    }

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_server(handle).await {
            log::warn!("[MetricsServer] metrics server 启动失败: {}", e);
        }
    });

    let _ = SERVER_STARTED.set(());
}

async fn start_server(_app_handle: tauri::AppHandle) -> Result<()> {
    let mut addr: SocketAddr = std::env::var("DSTU_METRICS_ADDR")
        .unwrap_or_else(|_| DEFAULT_METRICS_ADDR.to_string())
        .parse()
        .map_err(|e| anyhow::anyhow!("解析DSTU_METRICS_ADDR失败: {}", e))?;

    // Security: only allow loopback addresses
    if !addr.ip().is_loopback() {
        log::warn!(
            "[MetricsServer] Refusing to bind to non-loopback address {}. Using localhost instead.",
            addr
        );
        addr = SocketAddr::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            addr.port(),
        );
    }

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("metrics server绑定失败: {}", e))?;

    log::info!(
        "[MetricsServer] Metrics server listening on http://{}",
        addr
    );

    loop {
        let (stream, _) = listener
            .accept()
            .await
            .map_err(|e| anyhow::anyhow!("metrics server accept错误: {}", e))?;
        let io = TokioIo::new(stream);
        tokio::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(handle_request))
                .await
            {
                log::warn!("[MetricsServer] connection error: {}", err);
            }
        });
    }
}

async fn handle_request(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/metrics") => {
            let _guard = METRICS_GATHER_GUARD.lock().await;
            let payload = gather_metrics();
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/plain; version=0.0.4")
                .body(Full::new(Bytes::from(payload)))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("metrics response build failed"))));
            Ok(response)
        }
        _ => {
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Full::new(Bytes::from("Not Found")))
                .unwrap_or_else(|_| Response::new(Full::new(Bytes::from("Not Found"))));
            Ok(response)
        }
    }
}

fn gather_metrics() -> String {
    let mut sections = Vec::new();

    if let Some(queue_metrics) = crate::persistent_message_queue::export_queue_metrics() {
        sections.push(queue_metrics);
    }

    sections.join("\n")
}
