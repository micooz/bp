use std::fs;

const DEFAULT_LOG4RS_YAML: &str = "log4rs.yaml";

pub fn init() {
    let yaml_exists = fs::try_exists(DEFAULT_LOG4RS_YAML).unwrap();

    if !yaml_exists {
        let content = include_str!("assets/log4rs.yaml");
        fs::write(DEFAULT_LOG4RS_YAML, content).unwrap();
    }

    log4rs::init_file(DEFAULT_LOG4RS_YAML, Default::default()).unwrap();
}
