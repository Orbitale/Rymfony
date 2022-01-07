use std::path::PathBuf;
use crate::utils::project_directory::get_rymfony_project_directory;

pub(crate) fn rymfony_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".rymfony.pid")
}

pub(crate) fn get_caddy_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".caddy.pid")
}

pub(crate) fn rymfony_server_info_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("rymfony_server_info.json")
}

pub(crate) fn php_server_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".php_server.pid")
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn php_fpm_conf_ini_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("fpm-conf.ini")
}

pub(crate) fn get_php_process_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.fpm.log")
}

pub(crate) fn get_php_process_err_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("process.fpm.err")
}

pub(crate) fn get_php_access_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("server.php.access.log")
}

pub(crate) fn get_php_error_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("server.php.access.err")
}

pub(crate) fn get_http_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("http.log")
}

pub(crate) fn get_http_vhost_log_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join("log").join("http.vhost.log")
}

pub(crate) fn get_http_error_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("log")
        .join("http.err")
}

pub(crate) fn get_caddy_config_file() -> PathBuf {
    get_rymfony_project_directory().unwrap()
        .join("Caddyfile")
}
