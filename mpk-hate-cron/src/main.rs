use chrono::NaiveTime;
use mpk_cracow_api::tram_stop_info;
use mpk_cracow_api::Mode::Departure;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://mpkhate:password@localhost/mpkhate".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    // Run (embedded) migrations if they're not applied already.
    sqlx::migrate!().run(&pool).await?;

    let all_trips = tram_stop_info(57019, Departure).await?.actual;

    for trip in all_trips {
        let actual_time: Option<NaiveTime> = if let Some(i) = trip.actual_time {
            Some(NaiveTime::parse_from_str(i.as_str(), "%H:%M")?)
        } else {
            None
        };

        let planned_time = NaiveTime::parse_from_str(trip.planned_time.as_str(), "%H:%M")?;

        sqlx::query!(
            "INSERT INTO trips VALUES ($1, NOW(), $2, $3, $4, $5, $6, $7, $8, $9, $10, $11);",
            trip.trip_id,
            trip.actual_relative_time,
            actual_time,
            trip.direction,
            trip.mixed_time,
            trip.passageid,
            trip.pattern_text,
            planned_time,
            trip.route_id,
            trip.status.to_string(),
            trip.vehicle_id,
        )
        .execute(&pool)
        .await?;
    }

    Ok(())
}
