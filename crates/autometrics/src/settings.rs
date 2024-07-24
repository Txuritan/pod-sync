//! Customize the global settings for Autometrics.
//!
//! ```rust
//! use autometrics::settings::AutometricsSettings;
//!
//! AutometricsSettings::builder()
//!    .service_name("test_service")
//!   .init();
//! ```
//!
//! See [`AutometricsSettingsBuilder`] for more details on the available options.

use once_cell::sync::OnceCell;
use std::env;
use thiserror::Error;

pub(crate) static AUTOMETRICS_SETTINGS: OnceCell<AutometricsSettings> = OnceCell::new();

/// Load the settings configured by the user or use the defaults.
///
/// Note that attempting to set the settings after this function is called will panic.
#[allow(dead_code)]
pub(crate) fn get_settings() -> &'static AutometricsSettings {
    AUTOMETRICS_SETTINGS.get_or_init(|| AutometricsSettingsBuilder::default().build())
}

pub struct AutometricsSettings {
    pub(crate) service_name: String,
    pub(crate) repo_url: String,
    pub(crate) repo_provider: String,
}

impl AutometricsSettings {
    pub fn builder() -> AutometricsSettingsBuilder {
        AutometricsSettingsBuilder::default()
    }
}

#[derive(Debug, Default)]
pub struct AutometricsSettingsBuilder {
    pub(crate) service_name: Option<String>,
    pub(crate) repo_url: Option<String>,
    pub(crate) repo_provider: Option<String>,
}

impl AutometricsSettingsBuilder {
    /// All metrics produced by Autometrics have a label called `service.name`
    /// (or `service_name` when exported to Prometheus) attached to
    /// identify the logical service they are part of.
    ///
    /// You can set this here or via environment variables.
    ///
    /// The priority for where the service name is loaded from is:
    /// 1. This method
    /// 2. `AUTOMETRICS_SERVICE_NAME` (at runtime)
    /// 3. `OTEL_SERVICE_NAME` (at runtime)
    /// 4. `CARGO_PKG_NAME` (at compile time), which is the name of the crate defined in the `Cargo.toml` file.
    pub fn service_name(mut self, service_name: impl Into<String>) -> Self {
        self.service_name = Some(service_name.into());
        self
    }

    pub fn repo_url(mut self, repo_url: impl Into<String>) -> Self {
        self.repo_url = Some(repo_url.into());
        self
    }

    pub fn repo_provider(mut self, repo_provider: impl Into<String>) -> Self {
        self.repo_provider = Some(repo_provider.into());
        self
    }

    /// Set the global settings for Autometrics. This returns an error if the
    /// settings have already been initialized.
    ///
    /// Note: this function should only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// If the Prometheus exporter is enabled, this will also initialize it.
    pub fn try_init(self) -> Result<&'static AutometricsSettings, SettingsInitializationError> {
        let settings = self.build();

        let settings = AUTOMETRICS_SETTINGS
            .try_insert(settings)
            .map_err(|_| SettingsInitializationError::AlreadyInitialized)?;

        Ok(settings)
    }

    /// Set the global settings for Autometrics.
    ///
    /// Note: this function can only be called once and MUST be called before
    /// the settings are used by any other Autometrics functions.
    ///
    /// If the Prometheus exporter is enabled, this will also initialize it.
    ///
    /// ## Panics
    ///
    /// This function will panic if the settings have already been initialized.
    pub fn init(self) -> &'static AutometricsSettings {
        self.try_init().unwrap()
    }

    fn build(self) -> AutometricsSettings {
        #[allow(clippy::unwrap_or_default)]
        let repo_url = self
            .repo_url
            .or_else(|| env::var("AUTOMETRICS_REPOSITORY_URL").ok())
            .unwrap_or_else(|| env!("CARGO_PKG_REPOSITORY").to_string());

        AutometricsSettings {
            service_name: self
                .service_name
                .or_else(|| env::var("AUTOMETRICS_SERVICE_NAME").ok())
                .or_else(|| env::var("OTEL_SERVICE_NAME").ok())
                .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
            repo_provider: self
                .repo_provider
                .or_else(|| env::var("AUTOMETRICS_REPOSITORY_PROVIDER").ok())
                .or_else(|| {
                    AutometricsSettingsBuilder::determinate_repo_provider_from_url(Some(&repo_url))
                        .map(|s| s.to_string())
                })
                .unwrap_or_default(),
            repo_url,
        }
    }

    fn determinate_repo_provider_from_url(url: Option<&str>) -> Option<&'static str> {
        url.and_then(|url| {
            let lowered = url.to_lowercase();

            if lowered.contains("github.com") {
                Some("github")
            } else if lowered.contains("gitlab.com") {
                Some("gitlab")
            } else if lowered.contains("bitbucket.org") {
                Some("bitbucket")
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Error)]
pub enum SettingsInitializationError {
    #[error("Autometrics settings have already been initialized")]
    AlreadyInitialized,
}
