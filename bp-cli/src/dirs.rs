use dirs::home_dir;
use std::fs::create_dir_all;
use std::path::PathBuf;

pub struct Dirs;

impl Dirs {
    pub fn init() -> std::io::Result<()> {
        create_dir_all(Self::root())?;
        create_dir_all(Self::run())?;
        create_dir_all(Self::logs())?;
        Ok(())
    }

    // ~/.bp
    pub fn root() -> PathBuf {
        let mut dir = home_dir().unwrap();
        dir.push(".bp");
        dir
    }

    // ~/.bp/run
    pub fn run() -> PathBuf {
        let mut dir = Self::root();
        dir.push("run");
        dir
    }

    // ~/.bp/logs
    pub fn logs() -> PathBuf {
        let mut dir = Self::root();
        dir.push("logs");
        dir
    }

    // ~/.bp/logs/bp.log
    pub fn log_file() -> PathBuf {
        let mut dir = Self::logs();
        dir.push("bp.log");
        dir
    }

    // ~/.bp/run/daemon.out
    pub fn run_daemon_out() -> PathBuf {
        let mut dir = Self::run();
        dir.push("daemon.out");
        dir
    }

    // ~/.bp/run/daemon.err
    pub fn run_daemon_err() -> PathBuf {
        let mut dir = Self::run();
        dir.push("daemon.err");
        dir
    }

    // ~/.bp/run/pid
    pub fn run_pid() -> PathBuf {
        let mut dir = Self::run();
        dir.push("pid");
        dir
    }
}