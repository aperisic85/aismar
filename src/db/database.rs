use sqlx::PgPool;
use ais::messages::position_report::PositionReport;

pub async fn insert_position_report(pool: &PgPool, pos: PositionReport) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO ais_position_reports (message_type, mmsi, latitude, longitude) VALUES ($1, $2, $3, $4)",
        pos.message_type as i32,
        pos.mmsi as i64,
        pos.latitude.unwrap_or(0.0) as f64,
        pos.longitude.unwrap_or(0.0) as f64
    )
    .execute(pool)
    .await?;
    Ok(())
}
