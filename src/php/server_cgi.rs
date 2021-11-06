use crate::php::structs::PhpServerSapi;
use crate::utils::project_directory::get_rymfony_project_directory;
use std::fs::{File, create_dir_all};
use std::process::Command;
use std::process::Stdio;

pub(crate) fn start(php_bin: String, port: &u16) -> (PhpServerSapi, Command) {
    let mut command = Command::new(php_bin);

    let log_path = get_rymfony_project_directory().unwrap().join("log");

    if !log_path.is_dir() {
        create_dir_all(&log_path).expect("Could not create log directory for project");
    }

    let error_log_file = log_path.join("php-cgi.log");

    if !error_log_file.exists() {
        let file_result = File::create(&error_log_file);
        if file_result.is_err() {
            warn!(
                "Could not create php-cgi log file in {}",
                &error_log_file.to_str().unwrap()
            );
        }
    }

    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("-b")
        .arg(format!("127.0.0.1:{}", port.to_string()))
        .arg("-d")
        .arg(format!("error_log={}", error_log_file.to_str().unwrap()))
        .arg("-e");

    (PhpServerSapi::CGI, command)
}
