use std::fs::File;

use anyhow::{Error, Result};
use daemonize::Daemonize;

use crate::dirs::Dirs;

pub fn daemonize() -> Result<()> {
    let stdout = File::create(Dirs::run_daemon_out()).unwrap();
    let stderr = File::create(Dirs::run_daemon_err()).unwrap();

    let daemonize = Daemonize::new()
        .pid_file(Dirs::run_pid()) // Every method except `new` and `start`
        // .chown_pid_file(true) // is optional, see `Daemonize` documentation
        // .working_directory("/tmp") // for default behaviour.
        // .user("root")
        // .group("root") // Group name
        // .group(2) // or group id.
        // .umask(0o777) // Set umask, `0o027` by default.
        .stdout(stdout) // Redirect stdout to `/tmp/daemon.out`.
        .stderr(stderr); // Redirect stderr to `/tmp/daemon.err`.

    // .exit_action(|| println!("Executed before master process exits"))
    // .privileged_action(|| "Executed before drop privileges");

    daemonize.start().map(|_| ()).map_err(|err| Error::msg(err.to_string()))
}
