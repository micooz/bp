#![feature(path_try_exists)]
use std::{env, fs};

use cmd_lib::{run_cmd, run_fun};

fn main() {
    println!("cargo:rerun-if-changed=../bp-web/src");

    let npm = run_fun!(which npm).unwrap();

    if npm.is_empty() {
        panic!("please install Node.js before build bp-web");
    }

    env::set_current_dir("../bp-web").unwrap();

    if !fs::try_exists("node_modules").unwrap() {
        run_cmd!(npm install).unwrap();
    }

    run_cmd!(npm run build).unwrap();
}
