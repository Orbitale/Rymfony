use crate::utils::php_binaries;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;
use users::get_current_gid;
use users::get_current_uid;

// Check: https://www.php.net/manual/en/install.fpm.configuration.php

// Possible values: alert, error, warning, notice, debug
const FPM_DEFAULT_LOG_LEVEL: &str = "notice";

const FPM_DEFAULT_PORT: u16 = 65535;

// The placeholders between brackets {{ }} will be replaced with proper values.
const FPM_DEFAULT_CONFIG: &str = "
[global]
log_level = {{ log_level }}

; Output to stderr
error_log = /dev/fd/2

; This should be managed by Rymfony.
; This gives the advantage of keeping control over the process,
; and possibly retrieve logs too (since logs can be piped with fpm's stderr with current config)
daemonize = no

[www]
user = {{ uid }}
group = {{ gid }}

listen = 127.0.0.1:{{ port }}
listen.allowed_clients = 127.0.0.1

pm = dynamic
pm.max_children = 5
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
pm.status_path = /_fpm-status

; Output to stderr
php_admin_value[error_log] = /dev/fd/2
php_admin_flag[log_errors] = on

; Redirect stdout and stderr to main error log instead of /dev/null (default config for fastcgi)
catch_workers_output = yes

; This allows injecting custom env vars like with \"APP_ENV=dev rymfony serve\"
clear_env = no
";

pub(crate) fn start() {
    let php_bin = php_binaries::current();

    if php_bin.contains("-fpm") {
        start_fpm(php_bin);
    } else if php_bin.contains("-cgi") {
        start_cgi(php_bin);
    } else {
        start_native(php_bin);
    }
}

fn start_fpm(php_bin: String) {
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

fn start_cgi(php_bin: String) {
    println!("Todo: serve php-cgi");
}

fn start_native(php_bin: String) {
    println!("Todo: serve php-cgi");
}
