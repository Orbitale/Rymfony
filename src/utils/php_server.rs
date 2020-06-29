use crate::utils::php_binaries;
use crate::utils::php_server_fpm::start_fpm;

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

    if php_bin.contains("-fpm") && cfg!(not(target_family = "windows")) {
        start_fpm(php_bin);
    } else if php_bin.contains("-cgi") {
        start_cgi(php_bin);
    } else {
        start_native(php_bin);
    }
}

fn start_cgi(php_bin: String) {
    println!("Todo: serve php-cgi");
}

fn start_native(php_bin: String) {
    println!("Todo: serve php native");
}
