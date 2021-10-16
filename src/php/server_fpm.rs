#[cfg(not(target_family = "windows"))]
use {
    regex::Regex,
    regex::RegexBuilder,
    std::fmt,
    std::error::Error,
    std::fs::File,
    std::fs::read_to_string,
    std::fs::remove_file,
    std::io::prelude::*,
    std::process::Command,
    std::path::Path,
    std::process::Stdio,
    users::get_current_uid,
    crate::php::structs::PhpServerSapi,
    crate::utils::network::find_available_port,
    crate::utils::project_directory::get_rymfony_project_directory
};

use crate::php::php_server::PhpServer;
use std::process::Child;

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

error_log = {{ rymfony_project_dir }}/log/fpm.err.log

; This should be managed by Rymfony.
; This gives the advantage of keeping control over the process,
; and possibly retrieve logs too (since logs can be piped with fpm's stderr with current config)
daemonize = no
{{ systemd_enable }}systemd_interval = 0

[www]
; Don't touch this line unless you know what you are doing
listen = 127.0.0.1:{{ port }}
listen.allowed_clients = 127.0.0.1

pm = dynamic
pm.max_children = 5
pm.start_servers = 2
pm.min_spare_servers = 1
pm.max_spare_servers = 3
pm.status_path = /_fpm-status

; Output to stderr
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

    let mut port = find_available_port(FPM_DEFAULT_PORT);

    // This is how you check whether systemd is active.
    // @see https://www.freedesktop.org/software/systemd/man/sd_booted.html
    let systemd_support = Path::new("/run/systemd/system/").exists();

    let rymfony_project_path = get_rymfony_project_directory().unwrap();

    let config = FPM_DEFAULT_CONFIG
        .replace("{{ port }}", &port.to_string())
        .replace("{{ log_level }}", FPM_DEFAULT_LOG_LEVEL)
        .replace("{{ rymfony_project_dir }}", &rymfony_project_path.to_str().unwrap())
        .replace("{{ systemd_enable }}", if systemd_support { "" } else { ";" })
    ;

    let fpm_config_file_path = rymfony_project_path.join("fpm-conf.ini");

    if !fpm_config_file_path.exists() {
        let mut fpm_config_file = File::create(&fpm_config_file_path).unwrap();
        fpm_config_file
            .write_all(config.as_bytes())
            .expect("Could not write to php-fpm config file.");
        info!("Saved FPM config file at {}", fpm_config_file_path.to_str().unwrap());
    } else {
        // Read the file and search the port
        let mut content = read_to_string(&fpm_config_file_path).unwrap();

        let port_used = read_port(&content).unwrap_or(port);

        let port_checked = find_available_port(port_used);
        content = change_port(&content, &port_checked);
        port = port_checked;

        remove_file(&fpm_config_file_path).expect("Could not remove php-fpm config file");
        let mut fpm_config_file = File::create(&fpm_config_file_path).unwrap();
        fpm_config_file.write_all(content.as_bytes()).expect(
            format!(
                "Could not write to php-fpm config file {}.",
                &fpm_config_file_path.to_str().unwrap()
            )
            .as_str(),
        );
        info!("Rewrote FPM config file at {}", fpm_config_file_path.to_str().unwrap());
    }

    let pid_filename = format!("{}/fpm.pid", rymfony_project_path.to_str().unwrap());

    let mut command = Command::new(php_bin);
    command
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("--nodaemonize")
        .arg("--pid")
        .arg(pid_filename)
        .arg("--fpm-config")
        .arg(fpm_config_file_path.to_str().unwrap());

    if uid == 0 {
        command.arg("--allow-to-run-as-root");
        warn!("You are running Rymfony as root!");
        warn!("Be careful with permissions if your application has to manipulate the filesystem!")
    }

    if let Ok(child) = command.spawn() {
        info!("Running php-fpm with PID {}", child.id());

        return (PhpServer::new(port, PhpServerSapi::FPM), child);
    }

    panic!("Could not start php-fpm.");
}
#[cfg(not(target_family = "windows"))]
#[derive(Debug)]
struct ReadPortError(String);

#[cfg(not(target_family = "windows"))]
impl fmt::Display for ReadPortError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "An error occured: {}", self.0)
    }
}
#[cfg(not(target_family = "windows"))]
impl Error for ReadPortError {}

#[cfg(not(target_family = "windows"))]
fn read_port(content: &str) -> std::result::Result<u16, ReadPortError> {
    let re = RegexBuilder::new(r"^[ ]*listen[ ]?=[ ]?(.*)$")
        .multi_line(true)
        .build()
        .unwrap();
    let regex_port = Regex::new(r"^(?:(?:127\.0\.0\.1|localhost):)?(\d{1,5})").unwrap();

    let mut found = false;
    let mut read_port = "".to_string();
    for caps in re.captures_iter(content) {
        let captures = regex_port.captures(&caps[1]);
        if !captures.is_none() && !found {
            found = true;
            let capss = captures.unwrap();
            read_port = format!("{}", &capss[1]);
        }
    }
    if !found {
        return Err(ReadPortError("Unable to find php-fpm port".into()));
    }

    let port_num: u16 = read_port.parse().unwrap();
    Ok(port_num)
}

#[cfg(not(target_family = "windows"))]
fn change_port(original_content: &str, new_port: &u16) -> String {
    let re = RegexBuilder::new(r"^([ ]*listen[ ]?=[ ]?)(.*)$")
        .multi_line(true)
        .build()
        .unwrap();
    let regex_port = Regex::new(r"^((?:(?:127\.0\.0\.1|localhost):)?)(\d{1,5})").unwrap();

    let mut found = false;
    let mut content = original_content.to_string();
    for caps in re.captures_iter(original_content) {
        let captures = regex_port.captures(&caps[2]);
        if captures.is_none() || found {
            content = content.replace(&caps[0], format!(";{}", &caps[0]).as_str());
        }
        if !captures.is_none() && !found {
            found = true;
            let capss = captures.unwrap();
            content = content.replace(
                &caps[0],
                format!("{}{}{}", &caps[1], &capss[1], new_port.to_string()).as_str(),
            );
        }
    }
    if !found {
        content = format!("{}\nlisten = 127.0.0.1:{}", content, new_port.to_string());
    }

    content
}

#[cfg(not(target_family = "windows"))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_port_one_line() {
        let str = "listen=1245";
        let port = 2316;
        let result = change_port(&str, &port);
        assert_eq!(result.as_str(), "listen=2316");
    }
    #[test]
    fn change_port_multiple_line() {
        let str = "
        listen = 127.0.0.1:1245
        ";
        let port = 2316;
        let result = change_port(&str, &port);
        assert_eq!(
            result.as_str(),
            "
        listen = 127.0.0.1:2316
        "
        );
    }
    #[test]
    fn change_port_multiple_listen() {
        let str = "
listen = 127.0.0.1:1245
listen = 127.0.0.1:158
listen =localhost:18
        ";
        let port = 2316;
        let result = change_port(&str, &port);
        assert_eq!(
            result.as_str(),
            "
listen = 127.0.0.1:2316
;listen = 127.0.0.1:158
;listen =localhost:18
        "
        );
    }
    #[test]
    fn change_port_listen_socket() {
        let str = "
listen = /path/to/socket
        ";
        let port = 2316;
        let result = change_port(&str, &port);
        assert_eq!(
            result.as_str(),
            "
;listen = /path/to/socket
        \nlisten = 127.0.0.1:2316"
        );
    }
}
