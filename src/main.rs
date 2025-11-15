use std::{collections::HashMap, sync::Arc};

use axum::{response::Html, routing::get, Router};
use clap::Parser;
use config::{Config, DaoType, LogFormat};
use facts::{AppRouter, AppState, HashMapDao, MockedDao};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

mod config;
mod facts;

const TRACING_STARTUP_TARGET: &str = "startup";

#[tokio::main]
async fn main() {
    let args = Config::parse();

    let subscriber_builder = tracing_subscriber::fmt().with_max_level(args.logging.log_level);

    match args.logging.log_format {
        LogFormat::Default => subscriber_builder.init(),
        LogFormat::Json => subscriber_builder.json().init(),
        LogFormat::Pretty => subscriber_builder.pretty().init(),
    }

    info!(
        target : TRACING_STARTUP_TARGET,
        "Tracing subscriber started with log level {} and {:?} log format", args.logging.log_level, args.logging.log_format,
    );

    let bind_address = format!("{}:{}", args.runtime.bind_host, args.runtime.bind_port);
    let listener = TcpListener::bind(&bind_address)
        .await
        .inspect_err(|err| {
            error!(
                target : TRACING_STARTUP_TARGET,
                "Cannot bind to {bind_address}: {err}"
            );
        })
        .unwrap();
    info!(
        target : TRACING_STARTUP_TARGET,
        "Created listener at {bind_address}"
    );

    let state = AppState {
        dao: match args.dao.dao_type {
            DaoType::Mocked => {
                info!(target : TRACING_STARTUP_TARGET, "Using MockedDao");
                Arc::new(MockedDao {})
            }
            DaoType::HashMap => {
                info!(target : TRACING_STARTUP_TARGET, "Using HashMapDao");
                Arc::new(HashMapDao::new(HashMap::new()))
            }
        },
    };

    let router = Router::new()
        .layer(TraceLayer::new_for_http())
        .route(
            "/",
            get(|| async {
                (
                    axum::http::StatusCode::OK,
                    Html(
r#"
<html>

<h1>Facts</h1>
<h2>Fact number 1: About smoking</h2>
<p>
  The phrase "smoking kills" is a direct statement about the severe health risks of tobacco use</br>
  Smoking is a leading cause of preventable death globally, leading to cancer, heart disease, stroke, and lung diseases
  like emphysema
</p>

</html>
"#,
                    ),
                )
            }),
        )
        .nest("/api/facts", AppRouter::default().into())
        .with_state(state);
    info!(target : TRACING_STARTUP_TARGET, "Created router");

    info!(target : TRACING_STARTUP_TARGET, "Starting server");
    axum::serve(listener, router)
        .await
        .inspect_err(|err| {
            error!(
                target : TRACING_STARTUP_TARGET,
                "Failed to start server: {err}"
            );
        })
        .unwrap();
}
