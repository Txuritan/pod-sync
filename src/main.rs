mod extractor;
mod handlers;
mod models;
mod tasks;
mod utils;

mod config;
mod database;

use std::{sync::Arc, time::Duration};

use axum::{extract::FromRef, Router};
use axum_extra::extract::cookie::Key;
use axum_prometheus::PrometheusMetricLayer;
use metrics_exporter_prometheus::PrometheusHandle;
use tokio::net::TcpListener;
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::prelude::*;

use crate::{config::Config, database::Database, tasks::Task};

#[derive(Clone)]
struct SyncState {
    key: Key,
    pub(crate) db: Database,
    pub(crate) cfg: Arc<Config>,
}

impl SyncState {
    async fn new() -> anyhow::Result<Self> {
        let config = Config::load().await?;
        let config = Arc::new(config);

        let db = Database::new().await?;

        Ok(Self {
            key: Key::from(&config.cookie_key()?),
            db,
            cfg: config.clone(),
        })
    }

    #[cfg(test)]
    async fn new_test(pool: sqlx::SqlitePool) -> anyhow::Result<Self> {
        let config = Arc::new(Config::load_test()?);

        Ok(SyncState {
            key: Key::from(&config.cookie_key()?),
            db: Database::new_test(pool).await?,
            cfg: config.clone(),
        })
    }
}

impl FromRef<SyncState> for Key {
    fn from_ref(input: &SyncState) -> Self {
        input.key.clone()
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let state = SyncState::new().await?;

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    tracing::info!(public = %state.cfg.public_address, private = %state.cfg.private_address, "server started");

    tokio::try_join!(
        start_public_server(state.clone(), prometheus_layer),
        start_private_server(state.clone(), metric_handle),
    )?;

    Ok(())
}

async fn start_public_server(
    state: SyncState,
    prometheus_layer: PrometheusMetricLayer<'static>,
) -> anyhow::Result<()> {
    let addr = state.cfg.public_address;

    let app = Router::new()
        .merge(handlers::app(state.clone()))
        .layer((
            prometheus_layer,
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(10)),
        ));

    let deletion_handle = Task::spawn(tasks::deletion);
    let identification_handle = Task::spawn(tasks::identification);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("waiting for tasks to finish");

    deletion_handle.await.expect("async task panicked");
    identification_handle.await.expect("async task panicked");

    state.db.shutdown().await?;

    Ok(())
}

async fn start_private_server(state: SyncState, metric_handle: PrometheusHandle) -> anyhow::Result<()> {
    use axum::routing::get;

    use std::future::ready;

    let addr = state.cfg.private_address;

    let app = Router::new().route("/metrics", get(move || ready(metric_handle.render())));

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
