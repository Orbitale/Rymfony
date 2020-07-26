use clap::App;
use clap::SubCommand;
use std::fs;
use std::path::Path;

use crate::utils::stop_process;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stop").about("Stops a potentially running HTTP server")
}

pub(crate) fn stop() {
    if Path::new(".pid").exists() {
        let pid = fs::read_to_string(".pid").unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped server running with PID {}", pid);
        fs::remove_file(".pid").expect("Could not remove the PID file")
    } else {
        info!("Seems like server is not running");
    }
}
