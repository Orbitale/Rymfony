#[cfg(not(target_family = "windows"))]
use std::{env, fs::File, io::prelude::*, process::Command};

#[cfg(not(target_family = "windows"))]
use users::{get_current_gid, get_current_uid};

use crate::php::php_server::PhpServer;
#[cfg(not(target_family = "windows"))]
use crate::php::php_server::PhpServerSapi;
use std::process::Child;

// Possible values: alert, error, warning, notice, debug
#[cfg(not(target_family = "windows"))]
const FPM_DEFAULT_LOG_LEVEL: &str = "notice";

#[cfg(not(target_family = "windows"))]
const FPM_DEFAULT_PORT: u16 = 65535;

// The placeholders between brackets {{ }} will be replaced with proper values.
#[cfg(not(target_family = "windows"))]
const FPM_DEFAULT_CONFIG: &str = "
[global]
log_level = {{ log_level }}

; Output to stderr
error_log = /dev/fd/2

; This should be managed by Rymfony.
; This gives the advantage of keeping control over the process,
; and possibly retrieve logs too (since logs can be piped with fpm's stderr with current config)
daemonize = no
systemd_interval = 0

[www]
; Only works if launched as a root user
; TODO: check if this can be usable anyway
;user = {{ uid }}
;group = {{ gid }}

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

#[cfg(target_family = "windows")]
pub(crate) fn start(_php_bin: String) -> (PhpServer, Child) {
    panic!(
        "PHP-FPM does not exist on Windows.\
    It seems the PHP version you selected is wrong.\
    Please retry using a different version of PHP"
    );
}

#[cfg(not(target_family = "windows"))]
pub(crate) fn start(php_bin: String) -> (PhpServer, Child) {
    info!("Using php-fpm");

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

    let home_dir = env::var("HOME").unwrap();
    let fpm_config_filename = format!("{}/.rymfony/fpm-conf.ini", home_dir);
    let fpm_config_filename = fpm_config_filename.as_str();

    let mut file = File::create(fpm_config_filename).unwrap();
    file.write_all(config.as_bytes())
        .expect("Could not write to php-fpm config file.");

    let cwd = env::current_dir().unwrap();
    let pid_filename = format!("{}/.fpm.pid", cwd.to_str().unwrap());

    let mut command = Command::new(php_bin);
    command
        .arg("--nodaemonize")
        .arg("--pid")
        .arg(pid_filename)
        .arg("--fpm-config")
        .arg(fpm_config_filename);

    if let Ok(child) = command.spawn() {
        info!("Running php-fpm with PID {}", child.id());

        return (PhpServer::new(FPM_DEFAULT_PORT, PhpServerSapi::FPM), child);
    }

    panic!("Could not start php-fpm.");
}
