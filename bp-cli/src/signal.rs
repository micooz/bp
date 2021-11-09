use std::thread;

use signal_hook::{
    consts::{SIGINT, SIGTERM},
    iterator::Signals,
};

pub fn handle_signals() {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();

    thread::spawn(move || {
        for sig in signals.forever() {
            log::info!("received signal {:?}", sig);
            match sig {
                SIGINT | SIGTERM => {
                    // #[cfg(feature = "profile")]
                    // {
                    //     bp_cli::profile::set_prof_active(false);
                    //     bp_cli::profile::dump_profile();
                    // }
                    // TODO: gracefully exit the program
                    // exit(0);
                }
                _ => (),
            }
        }
    });
}
