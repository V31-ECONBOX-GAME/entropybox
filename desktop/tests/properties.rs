use desktop::constant::{keys, locations, loggers};
use desktop::properties::{ApplicationProperties, LoggingProperties};
use entropybox_starter_simulation::entropybox::config::{dir, load_from};
use std::path::Path;

const PROFILES: [&str; 3] = ["dev", "release", "test"];

fn application(profile: &str) -> ApplicationProperties {
    load_from(&dir(), locations::CONFIG_BASE_NAME, profile)
        .expect("properties load")
        .get(keys::APPLICATION)
        .expect("application section")
}

fn logging(profile: &str) -> LoggingProperties {
    load_from(&dir(), locations::CONFIG_BASE_NAME, profile)
        .expect("properties load")
        .get(keys::LOGGING)
        .expect("logging section")
}

#[test]
fn dir_points_inside_desktop() {
    assert!(dir().join("application.toml").exists());
}

#[test]
fn base_layer_supplies_every_field() {
    assert_eq!(application("none").name, "entropybox");
    assert_eq!(logging("none").level[loggers::ROOT], "info");
}

#[test]
fn the_application_name_does_not_move_between_profiles() {
    for profile in PROFILES {
        assert_eq!(application(profile).name, "entropybox");
    }
}

#[test]
fn a_profile_layer_only_moves_the_log_levels() {
    assert_eq!(logging("dev").level[loggers::ROOT], "debug");
    assert_eq!(logging("release").level[loggers::ROOT], "warn");
    assert_eq!(logging("test").level[loggers::ROOT], "error");
}

#[test]
fn every_profile_layer_parses() {
    assert!(
        dir()
            .join(format!("{}.toml", locations::CONFIG_BASE_NAME))
            .exists()
    );
    for profile in PROFILES {
        assert!(
            dir()
                .join(format!("{}-{profile}.toml", locations::CONFIG_BASE_NAME))
                .exists()
        );
        assert!(load_from(&dir(), locations::CONFIG_BASE_NAME, profile).is_ok());
    }
}

#[test]
fn missing_base_layer_is_an_error() {
    assert!(
        load_from(
            Path::new("/nonexistent"),
            locations::CONFIG_BASE_NAME,
            "dev"
        )
        .is_err()
    );
}
