#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate env_logger;
extern crate pretty_env_logger;
extern crate regex;
extern crate tokio;

mod config {
    pub(crate) mod config;
    pub(crate) mod paths;
}

mod commands {
    pub(crate) mod logs;
    pub(crate) mod new_symfony;
    pub(crate) mod php_list;
    pub(crate) mod serve;
    pub(crate) mod stop;
}

mod utils {
    pub(crate) mod current_process_name;
    pub(crate) mod network;
    pub(crate) mod project_directory;
    pub(crate) mod stop_process;
}

mod php {
    pub(crate) mod binaries;
    pub(crate) mod php_server;
    pub(crate) mod server_cgi;
    pub(crate) mod server_fpm;
    pub(crate) mod structs;
}

mod http {
    pub(crate) mod caddy;
    pub(crate) mod proxy_server;
}

use clap::Command as ClapCommand;
use clap::Arg;
use dirs::home_dir;
use log::Level;
use pretty_env_logger::env_logger::fmt::Color;
use pretty_env_logger::env_logger::fmt::Style;
use pretty_env_logger::env_logger::fmt::StyledValue;
use std::fmt;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use utils::current_process_name;

fn main() {
    let build_metadata = include_str!("../build_metadata.txt").trim().replace("\n", "");
    let build_metadata_str = build_metadata.as_str();

    let version = if build_metadata == "" { "dev" } else { build_metadata_str };

    let application_commands = vec![
        commands::logs::command_config(),
        commands::php_list::command_config(),
        commands::serve::command_config(),
        commands::stop::command_config(),
        commands::new_symfony::command_config(),
    ];

    let home_dir = home_dir().unwrap();
    if home_dir.to_str().unwrap() != "" {
        fs::create_dir_all(home_dir.join(".rymfony")).unwrap();
    }

    let app = ClapCommand::new("rymfony")
        .version(version)
        .author("Alex \"Pierstoval\" Rock <alex@orbitale.io>")
        .about("
A command-line tool to spawn a PHP server behind an HTTP FastCGI proxy,
inspired by Symfony CLI, but Open Source.

https://github.com/Orbitale/Rymfony
")
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .multiple_occurrences(true)
                .multiple_values(true)
                .takes_value(false)
                .help("Set the verbosity level. -v for debug, -vv for trace, -vvv to trace executed modules"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .takes_value(false)
                .help("Do not display any output. Has precedence over -v|--verbose"),
        )
        .subcommands(application_commands);

    let matches = app.get_matches();
    let verbose_value = matches.indices_of("verbose").unwrap_or_default();
    let is_quiet = matches.index_of("quiet").unwrap_or_default() > 0;

    set_verbosity_value(verbose_value.len(), is_quiet);

    let subcommand_name = matches.subcommand_name();

    match subcommand_name {
        Some("serve") => {
            crate::commands::serve::serve(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap())
        }
        Some("server:start") => {
            crate::commands::serve::serve(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap())
        }
        Some("stop") => crate::commands::stop::stop(),
        Some("new") => {
            crate::commands::new_symfony::new_symfony(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap())
        }
        Some("new:symfony") => crate::commands::new_symfony::new_symfony(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        Some("php:list") => crate::commands::php_list::php_list(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        Some("logs") => crate::commands::logs::logs(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        Some("log") => crate::commands::logs::logs(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        Some("local:server:log") => crate::commands::logs::logs(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        Some("server:log") => crate::commands::logs::logs(matches.subcommand_matches(&subcommand_name.unwrap()).unwrap()),
        _ => {
            // If no subcommand is specified,
            // re-run the program with "--help"
            let mut subprocess = Command::new(current_process_name::get().as_str())
                .arg("--help")
                .spawn()
                .expect("Failed to create the \"help\" command.");

            subprocess
                .wait()
                .expect("Failed to run the \"help\" command.");
        }
    };
}

fn set_verbosity_value(value: usize, is_quiet: bool) {
    let level = std::env::var("RYMFONY_LOG").unwrap_or(String::from("INFO"));
    let mut level = level.as_str();

    let mut builder = pretty_env_logger::formatted_timed_builder();

    if is_quiet {
        level = "OFF";
    } else {
        match value {
            1 => level = "DEBUG",           // -v
            v if v >= 2 => level = "TRACE", // -vv
            _ => {}
        }
    }

    builder
        .parse_filters(level)
        .format(move |f, record| {
            // This is the same format as the initial one in the pretty_env_logger crate,
            // but only the part with the module name is changed.

            let mut style = f.style();
            let level = colored_level(&mut style, record.level());

            let mut style = f.style();
            let target = if value > 2 {
                let target = format!(" {}", record.target());
                let max_width = max_target_width(&target);
                style.set_bold(true).value(Padded {
                    value: target,
                    width: max_width,
                })
            } else {
                style.value(Padded {
                    value: String::from(""),
                    width: 0,
                })
            };

            let time = f.timestamp_millis();

            writeln!(f, " {} {}{} > {}", time, level, target, record.args(),)
        })
        .try_init()
        .unwrap();
}

// This struct is a copy/paste of the one in pertty_env_logger.
// It's necessary for left-padding the message type.
struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: fmt::Display> fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value(" INFO"),
        Level::Warn => style.set_color(Color::Yellow).value(" WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
