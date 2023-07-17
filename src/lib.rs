#![doc = include_str!("../README.md")]
#![warn(
    clippy::all,
    missing_copy_implementations,
    missing_debug_implementations,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    trivial_numeric_casts,
    renamed_and_removed_lints,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]
#![deny(missing_docs)]

mod services;
mod update;

use std::sync::Mutex;

use once_cell::sync::Lazy;
use rocket::{
    catch, catchers,
    fairing::AdHoc,
    get, routes,
    serde::{json::Json, Deserialize, Serialize},
    Build, Request, Rocket,
};

use self::update::update_loop;

/// The global, concurrently accessible current status.
static STATUS: Lazy<Mutex<Option<Status>>> = Lazy::new(|| Mutex::new(None));

/// The configuration loaded additionally by Rocket.
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
struct Config {
    /// The service-specific configuration
    service: services::Config,
}

/// The current photovoltaic invertor status.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Status {
    /// The current power production (W).
    current_w: f32,
    /// The total energy produced since installation (kWh).
    total_kwh: f32,
    /// The (UNIX) timestamp of when the status was last updated.
    last_updated: u64,
}

/// An error used as JSON response.
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Error {
    /// The error message.
    error: String,
}

impl Error {
    /// Creates a new error result from a message.
    fn from(message: impl AsRef<str>) -> Self {
        let error = String::from(message.as_ref());

        Self { error }
    }
}

/// Returns the current (last known) status.
#[get("/", format = "application/json")]
async fn status() -> Result<Json<Status>, Json<Error>> {
    let status_guard = STATUS.lock().expect("Status mutex was poisoined");
    status_guard
        .map(Json)
        .ok_or_else(|| Json(Error::from("No status found (yet)")))
}

/// The version information as JSON response.
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct VersionInfo {
    /// The version of the build.
    version: String,
    /// The timestamp of the build.
    timestamp: String,
    /// The (most recent) git SHA used for the build.
    git_sha: String,
    /// The timestamp of the last git commit used for the build.
    git_timestamp: String,
}

impl VersionInfo {
    /// Retrieves the version information from the environment variables.
    fn new() -> Self {
        Self {
            version: String::from(env!("CARGO_PKG_VERSION")),
            timestamp: String::from(env!("VERGEN_BUILD_TIMESTAMP")),
            git_sha: String::from(&env!("VERGEN_GIT_SHA")[0..7]),
            git_timestamp: String::from(env!("VERGEN_GIT_COMMIT_TIMESTAMP")),
        }
    }
}

/// Returns the version information.
#[get("/version", format = "application/json")]
async fn version() -> Result<Json<VersionInfo>, Json<Error>> {
    Ok(Json(VersionInfo::new()))
}

/// Default catcher for any unsuppored request
#[catch(default)]
fn unsupported(status: rocket::http::Status, _request: &Request<'_>) -> Json<Error> {
    let code = status.code;

    Json(Error::from(format!(
        "Unhandled/unsupported API call or path (HTTP {code})"
    )))
}

/// Creates a Rocket and attaches the config parsing and update loop as fairings.
pub fn setup() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![status, version])
        .register("/", catchers![unsupported])
        .attach(AdHoc::config::<Config>())
        .attach(AdHoc::on_liftoff("Updater", |rocket| {
            Box::pin(async move {
                let config = rocket
                    .figment()
                    .extract::<Config>()
                    .expect("Invalid configuration");
                let service = services::get(config.service).expect("Invalid service");

                // We don't care about the join handle nor error results?
                let _service = rocket::tokio::spawn(update_loop(service));
            })
        }))
        .attach(AdHoc::on_liftoff("Version", |_| {
            Box::pin(async move {
                let name = env!("CARGO_PKG_NAME");
                let version = env!("CARGO_PKG_VERSION");
                let git_sha = &env!("VERGEN_GIT_SHA")[0..7];

                println!("☀️ Started {name} v{version} (git @{git_sha})");
            })
        }))
}
