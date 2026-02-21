use std::path::PathBuf;

use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::TcpListener;

use crate::api::{AppState, build_router};
use crate::config::watcher::trigger_hot_reload;
use crate::config::{AppConfig, SharedEngine};

/// Runs the Axum API server with graceful signal-driven shutdown.
pub async fn run_server(
    shared_engine: SharedEngine,
    app_config: AppConfig,
    config_path: PathBuf,
    filter_pack_dir: PathBuf,
    initial_filter_pack_digest: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let router = build_router(AppState {
        engine: shared_engine.clone(),
    });
    let bind_address = format!("{}:{}", app_config.server.host, app_config.server.port);
    let listener = bind_listener_with_backlog(&bind_address, 4096)?;

    tracing::info!(address = %bind_address, "arma server started");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(
            shared_engine,
            config_path,
            filter_pack_dir,
            initial_filter_pack_digest,
        ))
        .await?;
    Ok(())
}

fn bind_listener_with_backlog(
    bind_address: &str,
    backlog: i32,
) -> Result<TcpListener, Box<dyn std::error::Error>> {
    let address: std::net::SocketAddr = bind_address.parse()?;
    let domain = if address.is_ipv4() {
        Domain::IPV4
    } else {
        Domain::IPV6
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
    socket.set_reuse_address(true)?;
    socket.bind(&address.into())?;
    socket.listen(backlog)?;

    let std_listener: std::net::TcpListener = socket.into();
    std_listener.set_nonblocking(true)?;
    let listener = TcpListener::from_std(std_listener)?;
    Ok(listener)
}

async fn shutdown_signal(
    shared_engine: SharedEngine,
    config_path: PathBuf,
    filter_pack_dir: PathBuf,
    initial_filter_pack_digest: u64,
) {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(stream) => stream,
            Err(error) => {
                tracing::error!(error = %error, "failed to register SIGTERM handler");
                return;
            }
        };

        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(stream) => stream,
            Err(error) => {
                tracing::error!(error = %error, "failed to register SIGHUP handler");
                return;
            }
        };

        let mut active_filter_pack_dir = filter_pack_dir;
        let mut active_filter_pack_digest = initial_filter_pack_digest;
        loop {
            tokio::select! {
                _ = sigterm.recv() => {
                    tracing::info!("SIGTERM received; initiating graceful shutdown");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("CTRL-C received; initiating graceful shutdown");
                    break;
                }
                _ = sighup.recv() => {
                    tracing::info!("SIGHUP received; triggering manual hot-reload");
                    if let Some(updated_path) = trigger_hot_reload(
                        &shared_engine,
                        &config_path,
                        &active_filter_pack_dir,
                        &mut active_filter_pack_digest,
                    ).await {
                        active_filter_pack_dir = updated_path;
                    }
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(error = %error, "failed to register shutdown handler");
        }
    }
}
