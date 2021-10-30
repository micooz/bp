use bp_cli::dirs::Dirs;

#[test]
fn test_root() {
    assert!(Dirs::root().ends_with(".bp"));
}

#[test]
fn test_run() {
    assert!(Dirs::run().ends_with("run"));
}

#[test]
fn test_logs() {
    assert!(Dirs::logs().ends_with("logs"));
}

#[test]
fn test_log_file() {
    assert!(Dirs::log_file().ends_with("bp.log"));
}

#[test]
fn test_run_daemon_out() {
    assert!(Dirs::run_daemon_out().ends_with("daemon.out"));
}

#[test]
fn test_run_daemon_err() {
    assert!(Dirs::run_daemon_err().ends_with("daemon.err"));
}

#[test]
fn test_run_pid() {
    assert!(Dirs::run_pid().ends_with("pid"));
}
