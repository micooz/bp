use crate::dirs::Dirs;
use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            policy::compound::{roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger, CompoundPolicy},
            RollingFileAppender,
        },
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};

pub fn init() {
    let encoder = Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S%.3f)} {h({l})} [{M}] {m}{n}"));
    let console = ConsoleAppender::builder().encoder(encoder.clone()).build();

    let policy_trigger = SizeTrigger::new(10 * 1024 * 1024); // 10 MB
    let policy_roller = FixedWindowRoller::builder().build("bp.{}.log", 5).unwrap();
    let compound_policy = CompoundPolicy::new(Box::new(policy_trigger), Box::new(policy_roller));
    let rolling_file = RollingFileAppender::builder()
        .encoder(encoder)
        .build(Dirs::log_file(), Box::new(compound_policy))
        .unwrap();

    let builder = Config::builder()
        .appender(Appender::builder().build("console", Box::new(console)))
        .appender(Appender::builder().build("rolling_file", Box::new(rolling_file)));

    let root = Root::builder()
        .appender("console")
        .appender("rolling_file")
        .build(LevelFilter::Info);

    let config = builder.build(root).unwrap();

    log4rs::init_config(config).unwrap();
}
