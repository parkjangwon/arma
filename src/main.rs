mod api;
mod cli;
mod config;
mod core;
mod filter_pack;
mod logger;
mod metrics;
mod tui;

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::sync::RwLock;

use crate::api::run_server;
use crate::cli::{process, update, Cli, Commands};
use crate::config::{load_app_config, load_filter_pack, resolve_filter_pack_dir};
use crate::config::watcher::filter_pack_digest;
use crate::core::engine::FilterEngine;
use crate::metrics::RuntimeMetrics;
use crate::tui::{run_dashboard, DashboardInfo};

/// Runs the ARMA API server with hot-reload workers.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { daemon } => run_start(daemon),
        Commands::Stop => {
            process::stop_process()?;
            Ok(())
        }
        Commands::Restart { daemon } => {
            if let Err(error) = process::stop_process() {
                tracing::warn!(error = %error, "restart stop failed, continuing startup");
            }
            run_start(daemon)
        }
        Commands::Reload => {
            process::reload_process()?;
            Ok(())
        }
        Commands::Status => run_status_dashboard(),
        Commands::Manual => {
            print_manual();
            Ok(())
        }
        Commands::Update { yes } => {
            update::run_update(yes)?;
            Ok(())
        }
    }
}

fn run_start(daemon: bool) -> Result<(), Box<dyn std::error::Error>> {
    process::prepare_start(daemon)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    let server_result = runtime.block_on(async {
        let config_path = PathBuf::from("config.yaml");
        let app_config = load_app_config(&config_path).await?;
        let _logger_guard = logger::init_logger(&app_config.logging.level, &app_config.logging.path)?;

        tracing::info!(log_path = %app_config.logging.path, "logger initialized");

        let filter_pack_dir = resolve_filter_pack_dir(&app_config, &config_path);
        let filter_profile = app_config.filter_pack.profile.clone();
        let filter_pack = load_filter_pack(&filter_pack_dir, filter_profile.as_deref()).await?;
        let initial_filter_pack_digest = filter_pack_digest(&filter_pack);
        let initial_engine = FilterEngine::new(&filter_pack)?;

        let shared_engine = Arc::new(RwLock::new(initial_engine));
        let runtime_metrics = Arc::new(RuntimeMetrics::new(1024));

        let watcher_engine = Arc::clone(&shared_engine);
        let watcher_config_path = config_path.clone();
        let watcher_filter_pack_dir = filter_pack_dir.clone();
        tokio::spawn(async move {
            if let Err(error) = crate::config::watcher::run_hot_reload_worker(
                watcher_engine,
                watcher_config_path,
                watcher_filter_pack_dir,
                initial_filter_pack_digest,
            )
            .await
            {
                tracing::error!(error = %error, "hot-reload worker terminated");
            }
        });

        run_server(
            shared_engine,
            runtime_metrics,
            app_config,
            config_path,
            filter_pack_dir,
            initial_filter_pack_digest,
        )
        .await
    });

    let clear_result = process::clear_pid_file();
    match (server_result, clear_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(error), Ok(())) => Err(error),
        (Ok(()), Err(error)) => Err(Box::new(error)),
        (Err(error), Err(_)) => Err(error),
    }
}

fn run_status_dashboard() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let (version, last_updated) = runtime.block_on(async {
        let config_path = PathBuf::from("config.yaml");
        let config = load_app_config(&config_path).await?;
        let filter_pack_dir = resolve_filter_pack_dir(&config, &config_path);
        let filter_pack = load_filter_pack(&filter_pack_dir, config.filter_pack.profile.as_deref()).await?;
        Ok::<(String, String), Box<dyn std::error::Error>>((
            filter_pack.version,
            filter_pack.last_updated,
        ))
    })?;

    run_dashboard(DashboardInfo {
        version,
        status_active: process::is_active(),
        filter_pack_last_updated: last_updated,
    })?;
    Ok(())
}

fn print_manual() {
    println!("ARMA Manual");
    println!("  arma start [-d|--daemon]");
    println!("  arma stop");
    println!("  arma restart [-d|--daemon]");
    println!("  arma reload");
    println!("  arma status");
    println!("  arma manual");
    println!("  arma update [--yes]");
}
