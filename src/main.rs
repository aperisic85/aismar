mod ais;
mod client;
mod config;
mod db;
use dotenvy::dotenv;
use sqlx::PgPool;
use std::env;
use std::sync::Arc;

use crate::config::AisConfig;
use axum::{Json, Router, http::StatusCode, routing::get};
use sqlx::FromRow;

#[derive(serde::Serialize, FromRow)]
struct AisPosition {
    mmsi: i64,
    latitude: f64,
    longitude: f64,
    received_at: Option<sqlx::types::chrono::NaiveDateTime>,
}

async fn get_positions(pool: Arc<PgPool>) -> Json<Vec<AisPosition>> {
    // Query to get the positions from the database
    let positions = sqlx::query_as!(
        AisPosition, // The type to map the results to
        r#"
        SELECT mmsi, latitude, longitude, received_at
        FROM ais_position_reports
        LIMIT 10
        "#,
    )
    .fetch_all(&*pool) // Dereference the Arc to pass a reference to PgPool
    .await
    .unwrap();

    // Return the results as JSON
    Json(positions)
}

// Function to get the latest position per MMSI
async fn get_last_positions(
    pool: Arc<PgPool>,
) -> Result<Json<Vec<AisPosition>>, (StatusCode, String)> {
    let positions = sqlx::query_as!(
        AisPosition, // The type to map the results to
        r#"
        SELECT DISTINCT ON (mmsi) mmsi, latitude, longitude, received_at
        FROM ais_position_reports
        ORDER BY mmsi, received_at DESC;
        "#,
    )
    .fetch_all(&*pool) // Dereference the Arc to pass a reference to PgPool
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(positions)) // Return the results as JSON
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not found in .env");
    let pool = PgPool::connect(&database_url).await.unwrap();
    let pool = Arc::new(pool);
    // Create configuration with multiple endpoints
    let config = AisConfig {
        endpoints: vec![
            "192.168.55.161:4712".into(), // Labinstica
            "192.168.52.161:4712".into(), // VDG
            "192.168.61.161:4712".into(), // ucka
            "192.168.66.161:4712".into(), // osor
        ],
        ..Default::default()
    };

    // Instantiate the connection manager
    //let mut manager = AisConnectionManager::new(config);
    let mut client = client::AisClient::new(config);
    client.run(pool.clone()).await?;
    // Start the connection manager
    //manager.start().await?;

    // Define the Axum application with the route
    let app = Router::new()
        .route(
            "/positions",
            get({
                let pool = pool.clone(); // Clone the Arc to move into the closure
                move || async move { get_positions(pool).await }
            }),
        )
        .route(
            "/last_positions",
            get({
                let pool = pool.clone();
                move || async move { get_last_positions(pool).await }
            }),
        );

    // run our app with hyper, listening globally on port 3000

    let api_server = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    api_server.await.unwrap(); //start API server in async spawn
    // Wait for Ctrl+C signal to gracefully shut down
    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");

    // Shutdown the connection manager
    //manager.shutdown().await;
    client.shutdown().await;

    Ok(())
}
