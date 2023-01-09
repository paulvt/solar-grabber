//! The My Autarco service.
//!
//! It uses the private My Autarco API to login (and obtain the session cookies) and
//! to retrieve the energy data (using the session cookies).
//! See also: <https://my.autarco.com>

use reqwest::{Client, ClientBuilder, Url};
use rocket::async_trait;
use serde::Deserialize;
use url::ParseError;

use crate::{Status, Config};

/// The base URL of My Autarco site.
const BASE_URL: &str = "https://my.autarco.com";

/// The interval between data polls (in seconds).
const POLL_INTERVAL: u64 = 300;

/// Instantiates the My Autarco service.
pub(crate) fn service(config: Config) -> Result<Service, reqwest::Error> {
    let client = ClientBuilder::new().cookie_store(true).build()?;
    let service = Service { client, config };

    Ok(service)
}

/// The My Autarco service.
#[derive(Debug)]
pub(crate) struct Service {
    /// The client used to do API requests using a cookie jar.
    client: Client,
    /// The configuration used to access the API.
    config: Config,
}

/// Returns the login URL for the My Autarco site.
fn login_url() -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/auth/login"))
}

/// Returns an API endpoint URL for the given site ID and endpoint of the My Autarco site.
fn api_url(site_id: &str, endpoint: &str) -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/api/site/{site_id}/kpis/{endpoint}",))
}

/// The energy data returned by the energy API endpoint.
#[derive(Debug, Deserialize)]
struct ApiEnergy {
    /// Total energy produced today (kWh)
    // pv_today: u32,
    /// Total energy produced this month (kWh)
    // pv_month: u32,
    /// Total energy produced since installation (kWh)
    pv_to_date: u32,
}

///  The power data returned by the power API endpoint.
#[derive(Debug, Deserialize)]
struct ApiPower {
    /// Current power production (W)
    pv_now: u32,
}

#[async_trait]
impl super::Service for Service {
    /// The interval between data polls (in seconds).
    ///
    /// Autaurco processes provides information from the invertor every 5 minutes.
    fn poll_interval(&self) -> u64 {
        POLL_INTERVAL
    }

    /// Performs a login on the My Autarco site.
    ///
    /// It mainly stores the acquired cookie in the client's cookie jar. The login credentials come
    /// from the loaded configuration (see [`Config`]).
    async fn login(&self) -> Result<(), reqwest::Error> {
        let params = [
            ("username", &self.config.username),
            ("password", &self.config.password),
        ];
        let login_url = login_url().expect("valid login URL");

        self.client.post(login_url).form(&params).send().await?;

        Ok(())
    }

    /// Retrieves a status update from the API of the My Autarco site.
    ///
    /// It needs the cookie from the login to be able to perform the action. It uses both the `energy`
    /// and `power` endpoint to construct the [`Status`] struct.
    async fn update(&self, last_updated: u64) -> Result<Status, reqwest::Error> {
        // Retrieve the data from the API endpoints.
        let api_energy_url = api_url(&self.config.site_id, "energy").expect("valid API energy URL");
        let api_response = self.client.get(api_energy_url).send().await?;
        let api_energy: ApiEnergy = match api_response.error_for_status() {
            Ok(res) => res.json().await?,
            Err(err) => return Err(err),
        };

        let api_power_url = api_url(&self.config.site_id, "power").expect("valid API power URL");
        let api_response = self.client.get(api_power_url).send().await?;
        let api_power: ApiPower = match api_response.error_for_status() {
            Ok(res) => res.json().await?,
            Err(err) => return Err(err),
        };

        Ok(Status {
            current_w: api_power.pv_now,
            total_kwh: api_energy.pv_to_date,
            last_updated,
        })
    }
}
