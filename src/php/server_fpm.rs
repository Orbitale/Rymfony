#[cfg(not(target_family = "windows"))]
use std::{env, fs::File, io::prelude::*, process::Command, path::PathBuf};

#[cfg(not(target_family = "windows"))]
use users::{get_current_gid, get_current_uid};

use crate::php::php_server::PhpServer;
#[cfg(not(target_family = "windows"))]
use crate::{php::structs::PhpServerSapi};
#[cfg(not(target_family = "windows"))]
use crate::utils::network::find_available_port;
use std::process::Child;

#[cfg(not(target_family = "windows"))]
use wsl::is_wsl;

// Possible values: alert, error, warning, notice, debug
#[cfg(not(target_family = "windows"))]
const FPM_DEFAULT_LOG_LEVEL: &str = "notice";

#[cfg(not(target_family = "windows"))]
const FPM_DEFAULT_PORT: u16 = 60000;

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
{{ systemd }}systemd_interval = 0

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

    let port = find_available_port(FPM_DEFAULT_PORT);

    // TODO systemd support should be detected dynamically on Linux
    let systemd_support = !cfg!(target_os = "macos") && !is_wsl();

    let config = FPM_DEFAULT_CONFIG
        .replace("{{ uid }}", uid_str.as_str())
        .replace("{{ gid }}", gid_str.as_str())
        .replace("{{ port }}", &port.to_string())
        .replace("{{ log_level }}", FPM_DEFAULT_LOG_LEVEL)
        .replace("{{ systemd }}", if systemd_support { "" } else { ";" });

    let home = env::var("HOME").unwrap_or(String::from(""));

    let fpm_config_file_path;

    if home != "" {
        fpm_config_file_path = PathBuf::from(home.as_str())
            .join(".rymfony")
            .join("fpm-conf.ini");
    } else {
        panic!("Cannot find the \"HOME\" directory in which to write the php-fpm configuration file.");
    }

    let mut fpm_config_file = File::create(&fpm_config_file_path).unwrap();
    fpm_config_file.write_all(config.as_bytes())
        .expect("Could not write to php-fpm config file.");

    dbg!(&fpm_config_file);

    let cwd = env::current_dir().unwrap();
    let pid_filename = format!("{}/.fpm.pid", cwd.to_str().unwrap());

    let mut command = Command::new(php_bin);
    command
        .arg("--nodaemonize")
        .arg("--pid")
        .arg(pid_filename)
        .arg("--fpm-config")
        .arg(fpm_config_file_path.to_str().unwrap());

    if let Ok(child) = command.spawn() {
        info!("Running php-fpm with PID {}", child.id());

        return (PhpServer::new(port, PhpServerSapi::FPM), child);
    }

    panic!("Could not start php-fpm.");
}
