use chrono::NaiveTime;
use futures::{stream, StreamExt};
use itertools::Itertools;
use mpk_cracow_api::Mode::{Arrival, Departure};
use mpk_cracow_api::{fetch_all_tram_stops, tram_stop_info, StopInfoTrips};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;
//use tracing::{error, info};
use log::{error, info};

const CONCURRENT_REQUESTS: usize = 10;

fn generate_db_url() -> String {
    if let Ok(x) = std::env::var("DATABASE_URL") {
        x
    } else {
        let user = std::env::var("POSTGRES_USER").unwrap_or_else(|_| "mpkhate".to_string());
        let password = std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
        let ip = std::env::var("POSTGRES_IP").unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
        let database = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "mpkhate".to_string());

        format!("postgres://{}:{}@{}:{}/{}", user, password, ip, port, database)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let db_connection_str = generate_db_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can connect to database");

    // Run (embedded) migrations if they're not applied already.
    sqlx::migrate!().run(&pool).await?;

    info!("Fetching all tram stops.");
    let all_stops = fetch_all_tram_stops()
        .await?
        .into_iter()
        .map(|x| x.short_name)
        .collect_vec(); // better type hints in IntelliJ Rust than .collect()
    info!("Finished fetching all tram stops.");

    info!("Starting to fetch data about specific stops.");
    let all_trips = stream::iter(all_stops)
        .map(|tram_stop_id| async move {
            info!("Fetching data for stop {}", &tram_stop_id);
            let stop_info = tram_stop_info(tram_stop_id.as_str(), Departure).await;
            info!("Finished fetching data for stop {}", &tram_stop_id);

            stop_info
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    all_trips
        .for_each(|trips| async {
            match trips {
                Ok(trips) => if !trips.actual.is_empty() {
                    match process_data_from_trips(trips.actual, &pool).await {
                        Ok(_) => info!("Processing data for stop {} succeeded!", trips.stop_short_name),
                        Err(e) => error!("DB error: {}", e),
                    }
                } else {
                    info!("No future trips fetched.")
                },
                Err(e) => error!("Request error: {}", e),
            }
        })
        .await;

    info!("Finished the execution, closing!");
    Ok(())
}

async fn process_data_from_trips(
    all_trips: Vec<StopInfoTrips>,
    pool: &Pool<Postgres>,
) -> Result<(), Box<dyn std::error::Error>> {
    for trip in all_trips {
        let actual_time = if let Some(i) = trip.actual_time {
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
        .execute(pool)
        .await?;
    }

    Ok(())
}
