use crate::adapters::configurations::command_bus_config::command_bus_init;
use crate::adapters::configurations::db;
use crate::adapters::configurations::event_bus_config::event_bus_init;
use crate::adapters::configurations::query_bus_config::query_bus_init;
use crate::adapters::configurations::service_config::app_state_init;
use crate::adapters::configurations::settings::AppSettings;
use salvo::catcher::Catcher;
use salvo::http::{HeaderValue, StatusCode};
use salvo::prelude::{Json, TcpListener};
use salvo::server::ServerHandle;
use salvo::{handler, Depot, FlowCtrl, Listener, Request, Response, Router, Server, Service};
use serde_json::json;
use std::error::Error;
use tokio::signal;

mod adapters;
mod application;
mod domain;
mod shared_kernel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    start_initialization().await;
    Ok(())
}

async fn start_initialization() {
    let _ = AppSettings::init();
    db::init().await;

    // Initialization of internal buses
    event_bus_init().await;
    command_bus_init().await;
    query_bus_init().await;
    app_state_init().await;
}

async fn run_web_server() -> bool {
    let router = Router::new().get(hello);
    let service = Service::new(router).catcher(Catcher::default().hoop(handle_error));
    let acceptor = TcpListener::new("127.0.0.1:5800").bind().await;
    let server = Server::new(acceptor);
    let handle = server.handle();
    // Listen Shutdown Signal
    tokio::spawn(listen_shutdown_signal(handle));
    server.serve(service).await;
    true
}

#[handler]
async fn hello() -> &'static str {
    "Hello"
}

#[handler]
async fn handle_error(&self, _req: &Request, _depot: &Depot, res: &mut Response, ctrl: &mut FlowCtrl) {
    res.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
    let status = res.status_code.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    res.render(Json(json!({
        "code": status.as_u16(),
        "error": status.to_string()
    })));
    ctrl.skip_rest();
}


async fn listen_shutdown_signal(handle: ServerHandle) {
    // Wait Shutdown Signal
    let ctrl_c = async {
        // Handle Ctrl+C signal
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        // Handle SIGTERM on Unix systems
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(windows)]
    let terminate = async {
        // Handle Ctrl+C on Windows (alternative implementation)
        signal::windows::ctrl_c().expect("failed to install signal handler").recv().await;
    };

    // Wait for either signal to be received
    tokio::select! {
        _ = ctrl_c => println!("ctrl_c signal received"),
        _ = terminate => println!("terminate signal received"),
    }

    // Graceful Shutdown Server
    handle.stop_graceful(None);
}