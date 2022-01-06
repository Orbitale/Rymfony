use std::path::PathBuf;
use crate::utils::project_directory::get_rymfony_project_directory;

pub(crate) fn rymfony_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".rymfony.pid")
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

pub(crate) fn get_caddy_pid_file() -> PathBuf {
    get_rymfony_project_directory().unwrap().join(".caddy.pid")
}
