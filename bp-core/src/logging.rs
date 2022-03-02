use std::fs;

use anyhow::Result;
use log4rs::config::RawConfig;

const DEFAULT_LOG4RS_YAML: &str = "log4rs.yaml";

pub struct LoggingConfig {
    raw: RawConfig,
}

impl LoggingConfig {
    #[doc(hidden)]
    pub fn file_path(&self) -> Option<String> {
        let (appenders, _errors) = self.raw.appenders_lossy(&Default::default());
        let _file_appender = appenders.iter().find(|&item| item.name() == "file")?;
        // TODO: file_appender.path()
        None
    }
}

pub fn init() -> Result<LoggingConfig> {
    let yaml_exists = fs::try_exists(DEFAULT_LOG4RS_YAML)?;

    if !yaml_exists {
        let content = include_str!("assets/log4rs.yaml");
        fs::write(DEFAULT_LOG4RS_YAML, content)?;
    }

    let content = fs::read_to_string(DEFAULT_LOG4RS_YAML)?;
    let config = serde_yaml::from_str::<RawConfig>(&content)?;

    log4rs::init_raw_config(config.clone()).unwrap();

    Ok(LoggingConfig { raw: config })
}
