pub mod application;
pub mod logging;

pub use application::ApplicationProperties;
pub use logging::LoggingProperties;

use crate::constant::locations;
use entropybox_starter_simulation::entropybox::config::{Config, ConfigError};

pub fn load() -> Result<Config, ConfigError> {
    entropybox_starter_simulation::entropybox::config::load(locations::CONFIG_BASE_NAME)
}
