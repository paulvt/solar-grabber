//! The My Autarco service.
//!
//! It uses the private My Autarco API to login (and obtain the session cookies) and
//! to retrieve the energy data (using the session cookies).
//! See also: <https://my.autarco.com>.

use reqwest::{Client, ClientBuilder, StatusCode, Url};
use rocket::{async_trait, serde::Deserialize};
use url::ParseError;

use crate::{
    services::{Error, Result},
    Status,
};

/// The base URL of My Autarco site.
const BASE_URL: &str = "https://my.autarco.com";

/// The interval between data polls (in seconds).
const POLL_INTERVAL: u64 = 300;

/// The configuration necessary to access the My Autarco API.
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde")]
pub(crate) struct Config {
    /// The username of the account to login with
    username: String,
    /// The password of the account to login with
    password: String,
    /// The Autarco site ID to track
    site_id: String,
}

/// Instantiates the My Autarco service.
pub(crate) fn service(config: Config) -> Result<Service> {
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
#[serde(crate = "rocket::serde")]
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
#[serde(crate = "rocket::serde")]
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
    async fn login(&mut self) -> Result<()> {
        let login_url = login_url().expect("valid login URL");
        let params = [
            ("username", &self.config.username),
            ("password", &self.config.password),
        ];
        let response = self.client.post(login_url).form(&params).send().await?;
        match response.error_for_status() {
            Ok(_) => Ok(()),
            Err(e) if e.status() == Some(StatusCode::UNAUTHORIZED) => Err(Error::NotAuthorized),
            Err(e) => Err(e.into()),
        }
    }

    /// Retrieves a status update from the API of the My Autarco site.
    ///
    /// It needs the cookie from the login to be able to perform the action. It uses both the
    /// `energy` and `power` endpoint to construct the [`Status`] struct.
    async fn update(&mut self, last_updated: u64) -> Result<Status> {
        // Retrieve the data from the API endpoints.
        let api_energy_url = api_url(&self.config.site_id, "energy").expect("valid API energy URL");
        let api_response = self.client.get(api_energy_url).send().await?;
        let api_energy = match api_response.error_for_status() {
            Ok(res) => res.json::<ApiEnergy>().await?,
            Err(err) if err.status() == Some(StatusCode::UNAUTHORIZED) => {
                return Err(Error::NotAuthorized)
            }
            Err(err) => return Err(err.into()),
        };

        let api_power_url = api_url(&self.config.site_id, "power").expect("valid API power URL");
        let api_response = self.client.get(api_power_url).send().await?;
        let api_power = match api_response.error_for_status() {
            Ok(res) => res.json::<ApiPower>().await?,
            Err(err) => return Err(err.into()),
        };

        Ok(Status {
            current_w: api_power.pv_now as f32,
            total_kwh: api_energy.pv_to_date as f32,
            last_updated,
        })
    }
}
