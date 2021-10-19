use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::path::PathBuf;

pub async fn init() {
    let file_path = get_file_path();

    let encoder = Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l})} [{M}] {m}{n}"));
    let console = ConsoleAppender::builder().encoder(encoder.clone()).build();
    let file = FileAppender::builder()
        .encoder(encoder)
        .build(file_path.clone())
        .unwrap();

    let builder = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("file", Box::new(file)));

    let root = Root::builder()
        .appender("console")
        .appender("file")
        .build(LevelFilter::Info);

    let config = builder.build(root).unwrap();

    log4rs::init_config(config).unwrap();

    log::info!("log files are stored at {}", file_path.to_str().unwrap());
}

// return ~/.bp/logs/bp.log
fn get_file_path() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".bp");
    dir.push("logs");
    dir.push("bp.log");
    dir
}
