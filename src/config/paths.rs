use std::path::PathBuf;
use std::fs::File;
use crate::utils::project_directory::get_rymfony_project_directory;

pub(crate) fn rymfony_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".rymfony.pid")
}

pub(crate) fn get_caddy_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".caddy.pid")
}

pub(crate) fn php_server_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".php_server.pid")
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn php_fpm_conf_ini_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("fpm-conf.ini")
}

pub(crate) fn get_rymfony_process_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.rymfony.log")
}

pub(crate) fn get_rymfony_process_err_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.rymfony.err")
}

pub(crate) fn get_php_process_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.fpm.log")
}

pub(crate) fn get_php_process_err_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.fpm.err")
}

pub(crate) fn get_php_server_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("server.php.log")
}

pub(crate) fn get_php_server_error_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("server.php.err")
}

pub(crate) fn get_http_server_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("http.server.log")
}

pub(crate) fn get_http_vhost_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("http.vhost.log")
}

pub(crate) fn get_http_process_stderr_file() -> PathBuf {
    let path = get_rymfony_project_directory().unwrap()
        .join("log")
        .join("process.http.stderr");

    if !path.exists() { File::create(&path).expect("Could not create HTTP process stderr file."); }

    path
}

pub(crate) fn get_http_process_stdout_file() -> PathBuf {
    let path = get_rymfony_project_directory().unwrap()
        .join("log")
        .join("process.http.stdout");

    if !path.exists() { File::create(&path).expect("Could not create HTTP process stdout file."); }

    path
}

pub(crate) fn get_caddy_config_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("Caddyfile")
}

pub(crate) fn get_caddy_runtime_config_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("Caddyfile.runtime")
}
