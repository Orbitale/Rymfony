#[allow(unused_imports)]

#[macro_use] extern crate prettytable;
#[macro_use] extern crate log;
extern crate regex;
extern crate pretty_env_logger;
extern crate ctrlc;
extern crate httparse;

mod config {
    pub(crate) mod config;
}

mod commands {
    pub(crate) mod new_symfony;
    pub(crate) mod php_list;
    pub(crate) mod serve;
    pub(crate) mod stop;
}

mod utils {
    pub(crate) mod current_process_name;
    pub(crate) mod logger;
    pub(crate) mod network;
    pub(crate) mod stop_process;
}

mod php {
    pub(crate) mod binaries;
    pub(crate) mod php_server;
    pub(crate) mod server_cgi;
    pub(crate) mod server_fpm;
    pub(crate) mod server_native;
    pub(crate) mod structs;
}

mod http {
    pub(crate) mod fastcgi_handler;
    pub(crate) mod proxy_server;
    pub(crate) mod version;
}

use clap::App;
use clap::Arg;
use dirs::home_dir;
use std::fs;
use std::process::Command;
use utils::current_process_name;

fn main() {
    let home_dir = home_dir().unwrap();
    if home_dir.to_str().unwrap() != "" {
        fs::create_dir_all(home_dir.join(".rymfony")).unwrap();
    }

    let commands = vec![
        crate::commands::php_list::command_config(),
        crate::commands::serve::command_config(),
        crate::commands::stop::command_config(),
        crate::commands::new_symfony::command_config(),
    ];

    let app = App::new("rymfony")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .takes_value(false)
                .help("Set the verbosity level. -v for verbose, -vv for debug"),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .help("Do not display any output. Has precedence over -v|--verbose"),
        )
        .subcommands(commands);

    let matches = app.get_matches();
    let verbose_value = matches.indices_of("verbose").unwrap_or_default();
    let is_quiet = matches.index_of("quiet").unwrap_or_default() > 0;

    set_verbosity_value(verbose_value.len(), is_quiet);

    let subcommand_name = matches.subcommand_name();

    match subcommand_name {
        Some("serve") => {
            crate::commands::serve::serve(matches.subcommand_matches("serve").unwrap())
        }
        Some("server:start") => {
            crate::commands::serve::serve(matches.subcommand_matches("server:start").unwrap())
        }
        Some("stop") => crate::commands::stop::stop(),
        Some("new") => {
            crate::commands::new_symfony::new_symfony(matches.subcommand_matches("new").unwrap())
        }
        Some("new:symfony") => crate::commands::new_symfony::new_symfony(
            matches.subcommand_matches("new:symfony").unwrap(),
        ),
        Some("php:list") => crate::commands::php_list::php_list(),
        _ => {
            // If no subcommand is specified,
            // re-run the program with "--help"
            let mut subprocess = Command::new(current_process_name::get().as_str())
                .arg("--help")
                .spawn()
                .expect("Failed to start HTTP server");

            subprocess
                .wait()
                .expect("An error occured when trying to execute the HTTP server");
        }
    };
}

fn set_verbosity_value(value: usize, is_quiet: bool) {

    let level = std::env::var("RUST_LOG").unwrap_or(String::from("INFO"));
    let mut level = level.as_str();

    let mut builder = pretty_env_logger::formatted_timed_builder();

    if is_quiet {
        level = "OFF";
    } else {
        match value {
            1 => level = "DEBUG", // -v
            v if v >= 2 => level = "TRACE", // -vv
            _ => {},
        }
    }

    builder
        .parse_filters(level)
        .try_init()
        .unwrap();
}
