//! Module for handling the status updating/retrieval via the cloud service API.

use std::time::{Duration, SystemTime};

use rocket::tokio::time::sleep;

use crate::{
    services::{Error, Service, Services},
    STATUS,
};

/// The default sleep interval to use between checks.
const DEFAULT_SLEEP_INTERVAL: u64 = 10;

/// The sleep interval upper limit when applying exponential backoff.
const MAX_SLEEP_INTERVAL: u64 = 320;

/// The backoff factor.
const BACKOFF_FACTOR: f64 = 2.0;

/// Calculates the new interval by applying the backoff factor and taking the maximum into account.
fn back_off(interval: u64) -> u64 {
    let new_interval = (interval as f64 * BACKOFF_FACTOR) as u64;

    new_interval.min(MAX_SLEEP_INTERVAL)
}

/// Main update loop that logs in and periodically acquires updates from the API.
///
/// It updates the mutex-guarded current update [`Status`](crate::Status) struct which can be
/// retrieved via Rocket.
pub(super) async fn update_loop(service: Services) -> color_eyre::Result<()> {
    let mut service = service;

    // Log in on the cloud service.
    println!("⚡ Logging in...");
    service.login().await?;
    println!("⚡ Logged in successfully!");

    let mut last_updated = 0;
    let poll_interval = service.poll_interval();
    let mut sleep_interval = DEFAULT_SLEEP_INTERVAL;
    loop {
        // Wake up every 10 seconds and check if an update is due.
        sleep(Duration::from_secs(sleep_interval)).await;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if timestamp - last_updated < poll_interval {
            continue;
        }

        let status = match service.update(timestamp).await {
            Ok(status) => status,
            Err(Error::NotAuthorized) => {
                eprintln!("💥 Update unauthorized, trying to log in again...");
                if let Err(e) = service.login().await {
                    eprintln!("💥 Login failed: {e}; will retry in {sleep_interval} seconds...");
                    sleep_interval = back_off(sleep_interval);
                    continue;
                };
                println!("⚡ Logged in successfully!");
                sleep_interval = DEFAULT_SLEEP_INTERVAL;
                continue;
            }
            Err(e) => {
                eprintln!(
                    "💥 Failed to update status: {e}; will retry in {sleep_interval} seconds..."
                );
                sleep_interval = back_off(sleep_interval);
                continue;
            }
        };
        sleep_interval = DEFAULT_SLEEP_INTERVAL;
        last_updated = timestamp;

        println!("⚡ Updated status to: {status:#?}");
        let mut status_guard = STATUS.lock().expect("Status mutex was poisoned");
        status_guard.replace(status);
    }
}
