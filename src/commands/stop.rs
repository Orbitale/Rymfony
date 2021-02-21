use clap::App;
use clap::SubCommand;
use std::fs;

use crate::utils::project_directory::get_rymfony_project_directory;
use crate::utils::stop_process;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stop").about("Stops a potentially running HTTP server")
}

pub(crate) fn stop() {
    let project_folder =
        get_rymfony_project_directory().expect("Unable to get Rymfony folder for this project");
    let pid_path = project_folder.join(".pid");
    if pid_path.exists() {
        let pid = fs::read_to_string(&pid_path).unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped server running with PID {}", pid);
        fs::remove_file(&pid_path).expect("Could not remove the PID file")
    } else {
        info!("Seems like server is not running");
    }
}
