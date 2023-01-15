//! The Hoymiles service.
//!
//! It uses the private Hoymiles API to login (and obtain the session cookies) and
//! to retrieve the energy data (using the session cookies).
//! See also: <https://global.hoymiles.com>.

use std::sync::Arc;

use md5::{Digest, Md5};
use reqwest::{cookie::Jar as CookieJar, Client, ClientBuilder, Url};
use rocket::async_trait;
use serde::{Deserialize, Deserializer, Serialize};
use url::ParseError;

use crate::Status;

/// The base URL of Hoymiles API gateway.
const BASE_URL: &str = "https://global.hoymiles.com/platform/api/gateway";

/// The interval between data polls (in seconds).
const POLL_INTERVAL: u64 = 300;

/// The configuration necessary to access the Hoymiles.
#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    /// The username of the account to login with
    username: String,
    /// The password of the account to login with
    password: String,
    /// The ID of the Hoymiles station to track
    sid: u32,
}

/// Instantiates the Hoymiles service.
pub(crate) fn service(config: Config) -> Result<Service, reqwest::Error> {
    let cookie_jar = Arc::new(CookieJar::default());
    let client = ClientBuilder::new()
        .cookie_provider(Arc::clone(&cookie_jar))
        .build()?;
    let service = Service {
        client,
        config,
        cookie_jar,
    };

    Ok(service)
}

/// The Hoymiles service.
#[derive(Debug)]
pub(crate) struct Service {
    /// The client used to do API requests using a cookie jar.
    client: Client,
    /// The configuration used to access the API.
    config: Config,
    /// The cookie jar used for API requests.
    cookie_jar: Arc<CookieJar>,
}

/// Returns the login URL for the Hoymiles site.
fn login_url() -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/iam/auth_login"))
}

/// Returns an API endpoint URL for for the Hoymiles site.
fn api_url() -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/pvm-data/data_count_station_real_data"))
}

/// The request passed to the API login endpoint.
#[derive(Debug, Serialize)]
struct ApiLoginRequest {
    body: ApiLoginRequestBody,
}

impl ApiLoginRequest {
    /// Creates a new API login request.
    fn new(username: &str, password: &str) -> Self {
        let user_name = username.to_owned();
        let mut hasher = Md5::new();
        hasher.update(password.as_bytes());
        let password = format!("{:x}", hasher.finalize());

        let body = ApiLoginRequestBody {
            user_name,
            password,
        };

        Self { body }
    }
}

/// The request body passed to the API login endpoint.
#[derive(Debug, Serialize)]
struct ApiLoginRequestBody {
    password: String,
    user_name: String,
}

/// The response returned by the API login endpoint.
#[derive(Debug, Deserialize)]
struct ApiLoginResponse {
    // status: String,
    // message: String,
    /// The embedded response data
    data: ApiLoginResponseData,
    // systemNotice: Option<String>,
}

/// The response data returned by the API login endpoint.
#[derive(Debug, Deserialize)]
struct ApiLoginResponseData {
    /// The token to be used as cookie for API data requests.
    token: String,
}

/// The request passed to the API data endpoint.
#[derive(Debug, Serialize)]
struct ApiDataRequest {
    body: ApiDataRequestBody,
}

impl ApiDataRequest {
    /// Creates a new API data request.
    fn new(sid: u32) -> Self {
        let body = ApiDataRequestBody { sid };

        Self { body }
    }
}

/// The request body passed to the API data endpoint.
#[derive(Debug, Serialize)]
struct ApiDataRequestBody {
    sid: u32,
}

/// The response returned by the API data endpoint.
#[derive(Debug, Deserialize)]
struct ApiDataResponse {
    // status: String,
    // message: String,
    // /// The embedded response data
    data: ApiDataResponseData,
    // systemNotice: Option<String>,
}

/// Deserializes a string ([`&str`]) into a float ([`f32`]).
fn from_float_str<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    use serde::de::Error;

    let s = <&str>::deserialize(deserializer)?;
    s.parse::<f32>().map_err(D::Error::custom)
}

/// The response data returned by the API data endpoint.
#[derive(Debug, Deserialize)]
struct ApiDataResponseData {
    /// Energy produced today (Wh)
    #[serde(deserialize_with = "from_float_str")]
    today_eq: f32,
    // month_eq: f32,
    // year_eq: f32,
    /// Total energy produced since installation, excluding today's (Wh)
    #[serde(deserialize_with = "from_float_str")]
    total_eq: f32,
    /// Current power production
    #[serde(deserialize_with = "from_float_str")]
    real_power: f32,
    // co2_emission_reducation: f32,
    // plant_tree: u32,
    // data_time: String,
    // last_data_time: String,
    // capacitor: f32,
    // is_balance: bool,
    // is_reflux: bool,
    // reflux_station_data: Option<_>,
}

#[async_trait]
impl super::Service for Service {
    /// The interval between data polls (in seconds).
    ///
    /// Hoymiles processes provides information from the invertor every 15 minutes.
    fn poll_interval(&self) -> u64 {
        POLL_INTERVAL
    }

    /// Performs a login on the Hoymiles site.
    ///
    /// It mainly stores the acquired cookies in the client's cookie jar and adds the token cookie
    /// provided by the logins response. The login credentials come from the loaded configuration
    /// (see [`Config`]).
    async fn login(&self) -> Result<(), reqwest::Error> {
        let base_url = Url::parse(BASE_URL).expect("valid base URL");
        let login_url = login_url().expect("valid login URL");
        let login_request = ApiLoginRequest::new(&self.config.username, &self.config.password);
        let login_response = self
            .client
            .post(login_url)
            .json(&login_request)
            .send()
            .await?;
        let login_response_data = match login_response.error_for_status() {
            Ok(res) => res.json::<ApiLoginResponse>().await?.data,
            Err(err) => return Err(err),
        };
        // Insert the token in the reponse data as the cookie `hm_token` into the cookie jar.
        let cookie = format!("hm_token={}", login_response_data.token);
        self.cookie_jar.add_cookie_str(&cookie, &base_url);

        Ok(())
    }

    /// Retrieves a status update from the API of the Hoymiles site.
    ///
    /// It needs the cookies from the login to be able to perform the action.
    /// It uses a endpoint to construct the [`Status`] struct, but it needs to summarize the today
    /// value with the total value because Hoymiles only includes it after the day has finished.
    async fn update(&self, last_updated: u64) -> Result<Status, reqwest::Error> {
        let api_url = api_url().expect("valid API power URL");
        let api_data_request = ApiDataRequest::new(self.config.sid);
        let api_response = self
            .client
            .post(api_url)
            .json(&api_data_request)
            .send()
            .await?;
        let api_data = match api_response.error_for_status() {
            Ok(res) => res.json::<ApiDataResponse>().await?.data,
            Err(err) => return Err(err),
        };
        let current_w = api_data.real_power;
        let total_kwh = (api_data.total_eq + api_data.today_eq) / 1000.0;

        Ok(Status {
            current_w,
            total_kwh,
            last_updated,
        })
    }
}
