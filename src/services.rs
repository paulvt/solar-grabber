//! The supported cloud services.

pub(crate) mod hoymiles;
pub(crate) mod my_autarco;

use enum_dispatch::enum_dispatch;
use rocket::async_trait;
use serde::Deserialize;

use crate::Status;

/// The service-specific configuration necessary to access a cloud service API.
#[derive(Debug, Deserialize)]
#[serde(crate = "rocket::serde", tag = "kind")]
pub(crate) enum Config {
    /// Hoymiles (<https://global.hoymiles.com>)
    Hoymiles(hoymiles::Config),
    /// My Autarco (<https://my.autarco.com>)
    MyAutarco(my_autarco::Config),
}

/// Retrieves the service for the provided name (if supported).
pub(crate) fn get(config: Config) -> color_eyre::Result<Services> {
    match config {
        Config::Hoymiles(config) => Ok(Services::Hoymiles(hoymiles::service(config)?)),
        Config::MyAutarco(config) => Ok(Services::MyAutarco(my_autarco::service(config)?)),
    }
}

/// The supported cloud services.
#[enum_dispatch(Service)]
pub(crate) enum Services {
    /// Hoymiles (<https://global.hoymiles.com>)
    Hoymiles(hoymiles::Service),
    /// My Autarco (<https://my.autarco.com>)
    MyAutarco(my_autarco::Service),
}

/// Functionality trait of a cloud service.
#[async_trait]
#[enum_dispatch]
pub(crate) trait Service {
    /// The interval between data polls (in seconds).
    fn poll_interval(&self) -> u64;

    /// Perfoms a login on the cloud service (if necessary).
    async fn login(&mut self) -> Result<(), reqwest::Error>;

    /// Retrieves a status update using the API of the cloud service.
    async fn update(&mut self, timestamp: u64) -> Result<Status, reqwest::Error>;
}
