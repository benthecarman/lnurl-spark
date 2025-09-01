use axum::extract::DefaultBodyLimit;
use axum::http::Method;
use axum::routing::{get, post};
use axum::{http, Extension, Router};
use clap::Parser;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use nostr::Keys;
use spark::signer::DefaultSigner;
use spark_wallet::SparkWallet;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::config::*;
use crate::routes::*;

mod config;
mod models;
mod routes;

#[derive(Clone)]
pub struct State {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub keys: Keys,
    pub wallet: Arc<SparkWallet<DefaultSigner>>,

    // -- config options --
    pub domain: String,
    pub min_sendable: u64,
    pub max_sendable: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    pretty_env_logger::try_init()?;
    let config: Config = Config::parse();

    let keys = Keys::from_str(&config.nsec)?;

    let manager = ConnectionManager::<PgConnection>::new(config.pg_url.clone());
    let db_pool = Pool::builder()
        .max_size(10) // should be a multiple of 100, our database connection limit
        .test_on_check_out(true)
        .build(manager)
        .expect("Unable to build DB connection pool");

    let spark_config = config.spark_config();
    let signer = DefaultSigner::new(keys.secret_key().as_secret_bytes(), spark_config.network)?;
    let wallet = Arc::new(SparkWallet::connect(spark_config, signer).await?);

    let state = State {
        db_pool: db_pool.clone(),
        keys: keys.clone(),
        wallet,
        domain: config.domain,
        min_sendable: config.min_sendable,
        max_sendable: config.max_sendable,
    };

    let addr: std::net::SocketAddr = format!("{}:{}", config.bind, config.port)
        .parse()
        .expect("Failed to parse bind/port for webserver");

    println!("Webserver running on http://{addr}");

    let server_router = Router::new()
        .route("/health-check", get(health_check))
        .route("/get-invoice/:hash", get(get_invoice))
        .route("/verify/:desc_hash/:pay_hash", get(verify))
        .route("/.well-known/lnurlp/:name", get(get_lnurl_pay))
        .route("/v1/register", post(register_route))
        .fallback(fallback)
        .layer(Extension(state.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ]),
        )
        .layer(DefaultBodyLimit::max(1_000_000)); // max 1mb body size

    let server = axum::Server::bind(&addr).serve(server_router.into_make_service());

    // todo Invoice event stream for zaps

    let graceful = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to create Ctrl+C shutdown signal");
    });

    // Await the server to receive the shutdown signal
    if let Err(e) = graceful.await {
        eprintln!("shutdown error: {e}");
    }

    Ok(())
}
