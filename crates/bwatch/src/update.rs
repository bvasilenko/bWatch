use crate::BwatchError;
use bsuite_core::{SignedManifestUpdater, UpdateChannel, UpdateOutcome, Updater};
use semver::Version;
use std::io::Write as _;
use std::path::Path;

const UPDATE_BASE_URL_ENV: &str = "BSUITE_UPDATE_BASE_URL";
const UPDATE_BASE_URL_PLACEHOLDER: &str = "https://updates.example.invalid/bwatch/v1";

pub fn run(install_dir: &Path) -> Result<(), BwatchError> {
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))
        .expect("CARGO_PKG_VERSION is always valid semver");
    let base_url = std::env::var(UPDATE_BASE_URL_ENV)
        .unwrap_or_else(|_| UPDATE_BASE_URL_PLACEHOLDER.to_owned());
    let channel = UpdateChannel::new(base_url);
    let updater = SignedManifestUpdater::new().map_err(|e| {
        let _ = writeln!(std::io::stderr(), "update initialisation failed: {e}");
        BwatchError::Core(e)
    })?;
    let outcome = updater.check(&current_version, &channel).map_err(|e| {
        let _ = writeln!(std::io::stderr(), "update check failed: {e}");
        BwatchError::Core(e)
    })?;

    match &outcome {
        UpdateOutcome::UpToDate => {
            let _ = writeln!(std::io::stderr(), "already at the latest version");
        }
        UpdateOutcome::UpgradeAvailable { manifest, .. } => {
            let _ = writeln!(
                std::io::stderr(),
                "upgrading to version {}",
                manifest.version
            );
            updater.apply(&outcome, install_dir).map_err(|e| {
                let _ = writeln!(std::io::stderr(), "update apply failed: {e}");
                BwatchError::Core(e)
            })?;
            let _ = writeln!(std::io::stderr(), "upgrade complete");
        }
    }

    Ok(())
}
