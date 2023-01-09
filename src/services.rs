//! The supported cloud services.

pub(crate) mod my_autarco;

use enum_dispatch::enum_dispatch;
use rocket::async_trait;

use crate::{Status, Config};

/// Retrieves the service for the provided name (if supported).
pub(crate) fn get(service: &str, config: Config) -> color_eyre::Result<Services> {
    match service {
        "my_autarco" => Ok(Services::MyAutarco(my_autarco::service(config)?)),
        _ => panic!("Unsupported service: {service}"),
    }
}

/// The supported cloud services.
#[enum_dispatch(Service)]
pub(crate) enum Services {
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
    async fn login(&self) -> Result<(), reqwest::Error>;

    /// Retrieves a status update using the API of the cloud service.
    async fn update(&self, timestamp: u64) -> Result<Status, reqwest::Error>;
}
