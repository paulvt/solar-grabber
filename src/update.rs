//! Module for handling the status updating/retrieval via the cloud service API.

use std::time::{Duration, SystemTime};

use reqwest::StatusCode;
use rocket::tokio::time::sleep;

use crate::{
    services::{Service, Services},
    STATUS,
};

/// Main update loop that logs in and periodically acquires updates from the API.
///
/// It updates the mutex-guarded current update [`Status`](crate::Status) struct which can be
/// retrieved via Rocket.
pub(super) async fn update_loop(service: Services) -> color_eyre::Result<()> {
    // Log in on the cloud service.
    println!("⚡ Logging in...");
    service.login().await?;
    println!("⚡ Logged in successfully!");

    let mut last_updated = 0;
    let poll_interval = service.poll_interval();
    loop {
        // Wake up every 10 seconds and check if an update is due.
        sleep(Duration::from_secs(10)).await;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if timestamp - last_updated < poll_interval {
            continue;
        }

        let status = match service.update(timestamp).await {
            Ok(status) => status,
            Err(e) if e.status() == Some(StatusCode::UNAUTHORIZED) => {
                println!("✨ Update unauthorized, trying to log in again...");
                service.login().await?;
                println!("⚡ Logged in successfully!");
                continue;
            }
            Err(e) => {
                println!("✨ Failed to update status: {}", e);
                continue;
            }
        };
        last_updated = timestamp;

        println!("⚡ Updated status to: {:#?}", status);
        let mut status_guard = STATUS.lock().expect("Status mutex was poisoned");
        status_guard.replace(status);
    }
}
