//! Logging properties.

use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct LoggingProperties {
    pub level: BTreeMap<String, String>,
}
