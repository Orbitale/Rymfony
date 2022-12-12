use crate::config::paths;
use crate::http::caddy::get_caddy_path;
use crate::http::caddy::CADDYFILE;
use std::fs::read_to_string;
use std::fs::write;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

#[derive(Clone)]
pub(crate) struct CaddyCommandInput {
    pub(crate) config: String,
}

/// Returns a tuple containing:
/// * The full Caddy command to be executed
/// * The Caddy configuration as a string (equivalent to a Caddyfile), to be sent to Caddy's STDIN fd
///
pub(crate) fn get_caddy_start_command(
    use_tls: bool,
    host_name: String,
    http_port: u16,
    php_port: u16,
    document_root: String,
    php_entrypoint_file: String,
    add_server_sign: bool,
    debug: bool,
) -> (Command, CaddyCommandInput) {
    let caddy_path = get_caddy_path();
    let mut caddy_command = Command::new(&caddy_path);
    let caddy_runtime_config_file = paths::get_caddy_runtime_config_file();

    let caddy_config_file = paths::get_caddy_config_file();

    // let stderr_file = File::create(paths::get_http_process_stderr_file()).expect("Could not open HTTP error file.");
    let mut std_file_options = OpenOptions::new();
    std_file_options.create(true).read(true).append(true).write(true);

    let stdout_file =
        std_file_options.open(paths::get_http_process_stdout_file()).expect("Could not open HTTP error file.");
    let stderr_file = open_stderr_file(&std_file_options);

    caddy_command
        .stdin(Stdio::piped())
        .stdout(stdout_file)
        .stderr(stderr_file)
        .arg("run")
        .arg("--watch")
        .arg("--adapter").arg("caddyfile")
        .arg("--config").arg(&caddy_runtime_config_file) // This makes Caddy use STDIN for config
    ;

    let config = {
        let mut config: String = if !caddy_config_file.exists() {
            write(&caddy_config_file, CADDYFILE).expect("Could not write base Caddyfile config.");
            debug!("Wrote Caddy config to {}", &caddy_config_file.to_str().unwrap());

            CADDYFILE.to_string()
        } else {
            debug!("Reusing Caddy config from {}", &caddy_config_file.to_str().unwrap());

            read_to_string(&caddy_config_file).expect("Could not read base Caddyfile config file.")
        };

        config = config
            .replace("{{ debug }}", if debug { "" } else { "#" })
            .replace("{{ document_root }}", document_root.as_str())
            .replace("{{ host }}", &host_name)
            .replace("{{ server_port }}", &http_port.to_string())
            .replace("{{ https_port }}", &http_port.to_string())
            .replace("{{ show_http_port }}", if use_tls { "#" } else { "" })
            .replace("{{ log_file }}", paths::get_http_server_log_file().to_str().unwrap())
            .replace("{{ log_level }}", if debug { "DEBUG" } else { "INFO" })
            .replace("{{ php_entrypoint_file }}", php_entrypoint_file.as_str())
            .replace("{{ php_port }}", &php_port.to_string())
            .replace("{{ protocol }}", if use_tls { "" } else { "http://" })
            .replace("{{ use_tls }}", if use_tls { "" } else { "#" })
            .replace("{{ vhost_log_file }}", paths::get_http_vhost_log_file().to_str().unwrap())
            .replace("{{ with_server_sign }}", if add_server_sign { "" } else { "#" })
            .replace("{{ without_server_sign }}", if add_server_sign { "#" } else { "" });

        trace!("Final Caddy config:\n{}\n", &config);

        write(&caddy_runtime_config_file, &config).expect("Could not write runtime Caddyfile config.");

        config
    };

    (caddy_command, CaddyCommandInput { config })
}

pub(crate) fn start_caddy(caddy_command: &mut Command, caddy_config: String) -> Child {
    // let stderr_file = File::create(paths::get_http_process_stderr_file()).expect("Could not open HTTP error file.");
    let mut std_file_options = OpenOptions::new();
    std_file_options.create(true).read(true).append(true).write(true);

    let stderr_file = open_stderr_file(&std_file_options);

    let mut caddy_child_process = caddy_command.spawn().expect("Could not start HTTP server.");

    let caddy_pid = caddy_child_process.id().to_string();

    write(paths::get_caddy_pid_file(), caddy_pid).expect("Could not write PHP server PID to file.");

    let caddy_stdin = caddy_child_process.stdin.as_mut().unwrap();
    caddy_stdin.write_all(caddy_config.as_bytes()).expect("Could not write server config to Caddy STDIN.");

    let file_pointer = stderr_file.metadata().unwrap().len();

    let process_status = caddy_child_process.try_wait();

    match process_status {
        Ok(Some(status)) => {
            let exit_code = status.code().unwrap();

            // let stderr = String::from_utf8(output.stderr).unwrap();
            let stderr = "".to_string();

            if stderr.contains("listen tcp :80: bind: permission denied") {
                error!("Caddy is unable to listen to port 80, which is used for HTTP to HTTPS redirection.");
                error!("This can happen when you run Caddy (and therefore Rymfony) as non-root user.");
                error!("To make it work, you need to give Caddy the necessary network capabilities.");

                #[cfg(target_os = "linux")]
                {
                    let caddy_path = get_caddy_path();

                    error!(
                        "On most linux distributions, you can do it by running this command (possibly with \"sudo\"):"
                    );
                    error!("   setcap cap_net_bind_service=+ep {}", caddy_path.to_str().unwrap());
                }
            }

            if stderr == "".to_string() {
                let mut bytes = Vec::new();
                let mut stderr_file = open_stderr_file(&std_file_options);
                stderr_file.seek(SeekFrom::Start(file_pointer)).unwrap();
                stderr_file.read_to_end(&mut bytes).unwrap();
                let bytes_string = String::from_utf8(bytes.into()).unwrap();

                error!("Caddy failed to start with exit code {} and following error:", exit_code);
                let error_lines = bytes_string.trim().split("\n");
                for line in error_lines {
                    error!("  {}", line)
                }
            } else {
                unreachable!();
            }
        },
        Ok(None) => {
            info!("Running Caddy HTTP server");
        },
        Err(e) => panic!("An error occured when checking HTTP server status: {:?}", e),
    };

    caddy_child_process
}

fn open_stderr_file(file_options: &OpenOptions) -> File {
    file_options.open(paths::get_http_process_stderr_file()).expect("Could not open HTTP error file.")
}
