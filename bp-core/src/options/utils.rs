use std::{fs, io::Read};

use anyhow::{Error, Result};

pub fn options_from_file<T: serde::de::DeserializeOwned>(file: &str) -> Result<T> {
    let mut raw_str = String::new();
    let mut fd = fs::OpenOptions::new().read(true).open(file)?;
    fd.read_to_string(&mut raw_str)?;

    if file.ends_with(".yml") || file.ends_with(".yaml") {
        return from_yaml_str::<T>(&raw_str);
    }

    if file.ends_with(".json") {
        return from_json_str::<T>(&raw_str);
    }

    Err(Error::msg("invalid file format"))
}

fn from_yaml_str<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_yaml::from_str(s).map_err(|err| Error::msg(format!("fail to load YAML config: {}", err)))
}

fn from_json_str<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(|err| Error::msg(format!("fail to load JSON config: {}", err)))
}
