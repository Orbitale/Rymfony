use crate::command_handling::CommandHandler;
use crate::config::paths;
use crate::utils::project_directory::clean_rymfony_runtime_files;
use crate::utils::stop_process;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::fs;
use std::process::ExitCode;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(ClapCommand::new("stop").about("Stops a potentially running HTTP server"), Box::new(execute))
}

pub(crate) fn execute(_args: &ArgMatches) -> ExitCode {
    stop_rymfony();
    stop_php_server();
    stop_http_server();
    clean_rymfony_runtime_files();

    ExitCode::from(0)
}

fn stop_rymfony() {
    let rymfony_pid_file = paths::rymfony_pid_file();
    if rymfony_pid_file.exists() {
        let pid = fs::read_to_string(&rymfony_pid_file).unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped Rymfony running with PID {}", pid);
        fs::remove_file(&rymfony_pid_file).expect("Could not remove Rymfony's PID file")
    } else {
        info!("Seems like Rymfony is not running");
    }
}

fn stop_php_server() {
    let php_pid_file = paths::php_server_pid_file();
    if php_pid_file.exists() {
        let pid = fs::read_to_string(&php_pid_file).unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped PHP server running with PID {}", pid);
        let remove_result = fs::remove_file(&php_pid_file);
        if remove_result.is_err() {
            info!("Seems like PHP server was not running or was stopped when I checked for its status");
        }
    } else {
        info!("Seems like PHP server is not running");
    }
}

fn stop_http_server() {
    let caddy_pid_file = paths::get_caddy_pid_file();
    if caddy_pid_file.exists() {
        let pid = fs::read_to_string(&caddy_pid_file).unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped Caddy HTTP server running with PID {}", pid);
        fs::remove_file(&caddy_pid_file).expect("Could not remove HTTP server's PID file")
    } else {
        info!("Seems like Caddy HTTP server is not running");
    }
}
