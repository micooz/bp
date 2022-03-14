use bp_cli::dirs::Dirs;

use crate::constants::DEFAULT_CONFIG_FILE;

pub fn find_config_path() -> String {
    let mut path = Dirs::root();
    path.push("config.json");

    if std::fs::try_exists(&path).unwrap() {
        return path.to_str().unwrap().to_string();
    }

    DEFAULT_CONFIG_FILE.to_string()
}
