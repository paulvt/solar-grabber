//! The Hoymiles service.
//!
//! It uses the private Hoymiles API to login (and obtain the session cookies) and
//! to retrieve the energy data (using the session cookies).
//! See also: <https://global.hoymiles.com>.

use std::sync::Arc;

use chrono::{DateTime, Local, TimeZone};
use md5::{Digest, Md5};
use reqwest::{cookie::Jar as CookieJar, Client, ClientBuilder, Url};
use rocket::async_trait;
use serde::{Deserialize, Deserializer, Serialize};
use url::ParseError;

use crate::{services::Result, Status};

/// The base URL of Hoymiles API gateway.
const BASE_URL: &str = "https://global.hoymiles.com/platform/api/gateway";

/// The date/time format used by the Hoymiles API.
const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// The language to switch the API to.
///
/// If not set, it seems it uses `zh_cn`.
const LANGUAGE: &str = "en_us";

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
pub(crate) fn service(config: Config) -> Result<Service> {
    let cookie_jar = Arc::new(CookieJar::default());
    let client = ClientBuilder::new()
        .cookie_provider(Arc::clone(&cookie_jar))
        .build()?;
    let total_kwh = 0f32;
    let service = Service {
        client,
        config,
        cookie_jar,
        total_kwh,
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
    /// The last known total produced energy value.
    total_kwh: f32,
}

/// Returns the login URL for the Hoymiles site.
fn login_url() -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/iam/auth_login"))
}

/// Returns an API endpoint URL for for the Hoymiles site.
fn api_url() -> Result<Url, ParseError> {
    Url::parse(&format!("{BASE_URL}/pvm-data/data_count_station_real_data"))
}

/// Captures JSON values that can either be a string or an object.
///
/// This is used for the API responses where the data field is either an object or an empty string
/// instead of `null`.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrObject<'a, T> {
    /// The value is an object (deserializable as type `T`).
    Object(T),
    /// The value is a string.
    String(&'a str),
}

/// Deserialize either a string or an object as an option of type `T`.
fn from_empty_str_or_object<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
    T: Deserialize<'de>,
{
    use serde::de::Error;

    match <StringOrObject<'_, T>>::deserialize(deserializer) {
        Ok(StringOrObject::String(s)) if s.is_empty() => Ok(None),
        Ok(StringOrObject::String(_)) => Err(Error::custom("Non-empty string not allowed here")),
        Ok(StringOrObject::Object(t)) => Ok(Some(t)),
        Err(err) => Err(err),
    }
}

/// Deserialize a string ([`&str`]) into a date/time ([`DateTime<Local>`]).
fn from_date_time_str<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    use serde::de::Error;

    let s = <&str>::deserialize(deserializer)?;
    Local
        .datetime_from_str(s, DATE_TIME_FORMAT)
        .map_err(D::Error::custom)
}

/// Deserializes a string ([`&str`]) into a float ([`f32`]).
///
/// This is used for the API responses where the value is a float put into a string.
fn from_float_str<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    use serde::de::Error;

    let s = <&str>::deserialize(deserializer)?;
    s.parse::<f32>().map_err(D::Error::custom)
}

/// Deserializes a string ([`&str`]) into an integer ([`u16`]).
///
/// This is used for the API responses where the value is an integer put into a string.
fn from_integer_str<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
    D::Error: serde::de::Error,
{
    use serde::de::Error;

    let s = <&str>::deserialize(deserializer)?;
    s.parse::<u16>().map_err(D::Error::custom)
}

/// The request passed to the API login endpoint.
#[derive(Debug, Serialize)]
struct ApiLoginRequest {
    /// The body of the API login request.
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
    /// The username to login with.
    password: String,
    /// The password to login with.
    user_name: String,
}

/// The response returned by the API login endpoint.
#[derive(Debug, Deserialize)]
struct ApiLoginResponse {
    /// The status (error) code as a string: 0 for OK, another number for error.
    #[serde(deserialize_with = "from_integer_str")]
    status: u16,
    /// The status message.
    message: String,
    /// The embedded response data.
    #[serde(deserialize_with = "from_empty_str_or_object")]
    data: Option<ApiLoginResponseData>,
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
    /// The body of the API data request.
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
    /// The ID of the Hoymiles station.
    sid: u32,
}

/// The response returned by the API data endpoint.
#[derive(Debug, Deserialize)]
struct ApiDataResponse {
    /// The status (error) code as a string: 0 for OK, another number for error.
    #[serde(deserialize_with = "from_integer_str")]
    status: u16,
    /// The status message.
    message: String,
    /// The embedded response data.
    #[serde(deserialize_with = "from_empty_str_or_object")]
    data: Option<ApiDataResponseData>,
    // systemNotice: Option<String>,
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
    #[serde(deserialize_with = "from_date_time_str")]
    last_data_time: DateTime<Local>,
    // capacitor: f32,
    // is_balance: bool,
    // is_reflux: bool,
    // reflux_station_data: Option<_>,
}

#[async_trait]
impl super::Service for Service {
    /// The interval between data polls (in seconds).
    ///
    /// Hoymiles processes information from the invertor about every 15 minutes. Since this is not
    /// really exact, we need to poll at a higher rate to detect changes faster!
    fn poll_interval(&self) -> u64 {
        POLL_INTERVAL
    }

    /// Performs a login on the Hoymiles site.
    ///
    /// It mainly stores the acquired cookies in the client's cookie jar and adds the token cookie
    /// provided by the logins response. The login credentials come from the loaded configuration
    /// (see [`Config`]).
    async fn login(&mut self) -> Result<()> {
        let base_url = Url::parse(BASE_URL).expect("valid base URL");
        let login_url = login_url().expect("valid login URL");

        // Insert the cookie `hm_token_language` to specific the API language into the cookie jar.
        let lang_cookie = format!("hm_token_language={}", LANGUAGE);
        self.cookie_jar.add_cookie_str(&lang_cookie, &base_url);

        let login_request = ApiLoginRequest::new(&self.config.username, &self.config.password);
        let login_response = self
            .client
            .post(login_url)
            .json(&login_request)
            .send()
            .await?;
        let login_response_data = match login_response.error_for_status() {
            Ok(res) => {
                let api_response = res.json::<ApiLoginResponse>().await?;
                eprintln!("api_response = {:#?}", &api_response);
                api_response.data.expect("No API response data found")
            }
            Err(err) => return Err(err.into()),
        };
        // Insert the token in the reponse data as the cookie `hm_token` into the cookie jar.
        let token_cookie = format!("hm_token={}", login_response_data.token);
        self.cookie_jar.add_cookie_str(&token_cookie, &base_url);

        Ok(())
    }

    /// Retrieves a status update from the API of the Hoymiles site.
    ///
    /// It needs the cookies from the login to be able to perform the action.
    /// It uses a endpoint to construct the [`Status`] struct, but it needs to summarize the today
    /// value with the total value because Hoymiles only includes it after the day has finished.
    async fn update(&mut self, _last_updated: u64) -> Result<Status> {
        let api_url = api_url().expect("valid API power URL");
        let api_data_request = ApiDataRequest::new(self.config.sid);
        let api_response = self
            .client
            .post(api_url)
            .json(&api_data_request)
            .send()
            .await?;
        let api_data = match api_response.error_for_status() {
            Ok(res) => {
                let api_response = res.json::<ApiDataResponse>().await?;
                eprintln!("api_response = {:#?}", &api_response);
                api_response.data.expect("No API response data found")
            }
            Err(err) => return Err(err.into()),
        };
        let current_w = api_data.real_power;
        let mut total_kwh = (api_data.total_eq + api_data.today_eq) / 1000.0;
        let last_updated = api_data.last_data_time.timestamp() as u64;

        // Sometimes it can be that `today_eq` is reset when the day switches but it has not been
        // added to `total_eq` yet. The `total_eq` should always be non-decreasing, so return the
        // last known value until this is corrected (this most suredly happens during the night).
        if total_kwh <= self.total_kwh {
            total_kwh = self.total_kwh
        } else {
            self.total_kwh = total_kwh;
        }

        Ok(Status {
            current_w,
            total_kwh,
            last_updated,
        })
    }
}
