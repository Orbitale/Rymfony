#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;
extern crate regex;

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

use crate::utils::logger::SimpleLogger;

use clap::App;
use log::LevelFilter;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use utils::current_process_name;

static LOGGER: SimpleLogger = SimpleLogger;

fn main() {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();

    let path = env::var("HOME").unwrap_or(String::from(""));

    if path != "" {
        let path = PathBuf::from(path).join("rymfony");
        fs::create_dir_all(path).unwrap();
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
        .subcommands(commands);

    let matches = app.get_matches();

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
