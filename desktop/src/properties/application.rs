//! Application properties.

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(default, deny_unknown_fields)]
pub struct ApplicationProperties {
    pub name: String,
}
