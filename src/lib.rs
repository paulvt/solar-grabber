#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links
)]
#![deny(missing_docs)]

mod services;
mod update;

use std::sync::Mutex;

use once_cell::sync::Lazy;
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::{get, routes, Build, Rocket};
use serde::{Deserialize, Serialize};

use self::update::update_loop;

/// The global, concurrently accessible current status.
static STATUS: Lazy<Mutex<Option<Status>>> = Lazy::new(|| Mutex::new(None));

/// The configuration loaded additionally by Rocket.
#[derive(Debug, Deserialize)]
struct Config {
    /// The service-specific configuration
    service: services::Config,
}

/// The current photovoltaic invertor status.
#[derive(Clone, Copy, Debug, Serialize)]
struct Status {
    /// Current power production (W)
    current_w: u32,
    /// Total energy produced since installation (kWh)
    total_kwh: u32,
    /// Timestamp of last update
    last_updated: u64,
}

/// Returns the current (last known) status.
#[get("/", format = "application/json")]
async fn status() -> Option<Json<Status>> {
    let status_guard = STATUS.lock().expect("Status mutex was poisoined");
    status_guard.map(Json)
}

/// Creates a Rocket and attaches the config parsing and update loop as fairings.
pub fn setup() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status])
        .attach(AdHoc::config::<Config>())
        .attach(AdHoc::on_liftoff("Updater", |rocket| {
            Box::pin(async move {
                let config = rocket
                    .figment()
                    .extract::<Config>()
                    .expect("Invalid configuration");
                let service = services::get(config.service).expect("Invalid service");

                // We don't care about the join handle nor error results?t
                let _ = rocket::tokio::spawn(update_loop(service));
            })
        }))
}
