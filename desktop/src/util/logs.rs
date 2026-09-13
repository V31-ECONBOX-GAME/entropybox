//! Logging properties translated into what LogPlugin takes.

use crate::constant::loggers;
use crate::properties::LoggingProperties;
use bevy::log::{Level, LogPlugin};
use bevy::prelude::default;

pub fn plugin(logging: &LoggingProperties) -> LogPlugin {
    LogPlugin {
        level: root_level(logging),
        filter: filter(logging),
        ..default()
    }
}

fn root_level(logging: &LoggingProperties) -> Level {
    logging
        .level
        .get(loggers::ROOT)
        .map(String::as_str)
        .map_or(Level::INFO, level)
}

fn filter(logging: &LoggingProperties) -> String {
    let directives = logging
        .level
        .iter()
        .filter(|(target, _)| target.as_str() != loggers::ROOT)
        .map(|(target, level)| format!("{target}={}", level.to_ascii_lowercase()))
        .collect::<Vec<_>>()
        .join(",");

    format!("{}{directives}", bevy::log::DEFAULT_FILTER)
}

fn level(level: &str) -> Level {
    match level.to_ascii_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn logging(pairs: &[(&str, &str)]) -> LoggingProperties {
        LoggingProperties {
            level: pairs
                .iter()
                .map(|(target, level)| ((*target).to_string(), (*level).to_string()))
                .collect(),
        }
    }

    #[test]
    fn root_becomes_the_plugin_level() {
        assert_eq!(
            root_level(&logging(&[(loggers::ROOT, "DEBUG")])),
            Level::DEBUG
        );
        assert_eq!(root_level(&logging(&[])), Level::INFO);
    }

    #[test]
    fn targets_become_filter_directives() {
        let directives = filter(&logging(&[("root", "info"), ("entropybox", "TRACE")]));

        assert!(directives.starts_with(bevy::log::DEFAULT_FILTER));
        assert!(directives.ends_with("entropybox=trace"));
        assert!(!directives.contains("root="));
    }
}
