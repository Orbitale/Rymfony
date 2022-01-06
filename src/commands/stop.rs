use clap::App;
use clap::SubCommand;
use std::fs;
use crate::config::paths;
use crate::utils::stop_process;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("stop").about("Stops a potentially running HTTP server")
}

pub(crate) fn stop() {
    stop_rymfony();
    stop_php_server();
    stop_http_server();
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
        fs::remove_file(&php_pid_file).expect("Could not remove PHP's PID file")
    } else {
        info!("Seems like PHP server is not running");
    }
}

fn stop_http_server() {
    let caddy_pid_file = paths::get_caddy_pid_file();
    if caddy_pid_file.exists() {
        let pid = fs::read_to_string(&caddy_pid_file).unwrap();
        stop_process::stop(pid.as_ref());
        info!("Stopped HTTP server running with PID {}", pid);
        fs::remove_file(&caddy_pid_file).expect("Could not remove HTTP server's PID file")
    } else {
        info!("Seems like HTTP server is not running");
    }
}