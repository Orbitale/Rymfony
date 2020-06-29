use crate::utils::php_binaries;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

#[cfg(not(target_family = "windows"))]
use users::{get_current_gid, get_current_uid};

#[cfg(target_family = "windows")]
pub(crate) fn start_fpm(php_bin: String) {
    panic!(
        "PHP-FPM does not exist on Windows.\
    It seems the PHP version you selected is wrong.\
    Please retry using a different version of PHP"
    );
}

#[cfg(not(target_family = "windows"))]
pub(crate) fn start_fpm(php_bin: String) {
    println!("Using php-fpm");

    let uid = get_current_uid();
    let uid_str = uid.to_string();

    let gid = get_current_gid();
    let gid_str = gid.to_string();

    let port = FPM_DEFAULT_PORT.to_string();

    let config = FPM_DEFAULT_CONFIG
        .replace("{{ uid }}", uid_str.as_str())
        .replace("{{ gid }}", gid_str.as_str())
        .replace("{{ port }}", port.as_str())
        .replace("{{ log_level }}", FPM_DEFAULT_LOG_LEVEL);

    let fpm_config_filename = "~/.rymfony/fpm-conf.ini";

    let mut file = File::create(fpm_config_filename).unwrap();
    file.write_all(config.as_bytes());

    let cwd = env::current_dir().unwrap();
    let pid_filename = format!("{}/.fpm.pid", cwd.to_str().unwrap());

    let mut command = Command::new(php_bin);
    command
        .arg("--pid")
        .arg(pid_filename)
        .arg("--fpm-config")
        .arg(fpm_config_filename);

    if let Ok(child) = command.spawn() {
        println!("Child's ID is {}", child.id());
    } else {
        println!("ls command didn't start");
    }

    //println!("PHP running in background with PID {}", php_command.id());
}
