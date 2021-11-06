use std::path::PathBuf;
use crate::utils::project_directory::get_rymfony_project_directory;

pub(crate) fn rymfony_pid_file() -> PathBuf {
    PathBuf::from(format!("{}/rymfony.pid", get_rymfony_project_directory().unwrap().to_str().unwrap()))
}

pub(crate) fn rymfony_server_info_file() -> PathBuf {
    PathBuf::from(format!("{}/rymfony_server_info", get_rymfony_project_directory().unwrap().to_str().unwrap()))
}

pub(crate) fn php_server_pid_file() -> PathBuf {
    PathBuf::from(format!("{}/php_server.pid", get_rymfony_project_directory().unwrap().to_str().unwrap()))
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn php_fpm_conf_ini_file() -> PathBuf {
    PathBuf::from(format!("{}/fpm-conf.ini", get_rymfony_project_directory().unwrap().to_str().unwrap()))
}
