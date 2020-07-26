#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;
extern crate regex;

mod commands {
    pub(crate) mod php_list;
    pub(crate) mod serve;
    pub(crate) mod stop;
}

mod utils {
    pub(crate) mod current_process_name;
    pub(crate) mod stop_process;
}

mod php {
    pub(crate) mod binaries;
    pub(crate) mod php_server;
    pub(crate) mod server_cgi;
    pub(crate) mod server_fpm;
    pub(crate) mod server_native;
}

mod http {
    pub(crate) mod proxy_server;
    pub(crate) mod fastcgi_handler;
}

use crate::commands::php_list::command_config as php_list_cmd;
use crate::commands::php_list::php_list;
use crate::commands::serve::command_config as serve_cmd;
use crate::commands::serve::serve;
use crate::commands::stop::command_config as stop_cmd;
use crate::commands::stop::stop;

use clap::App;
use std::fs;
use std::process::Command;
use utils::current_process_name;

fn main() {
    fs::create_dir_all("~/.rymfony/").unwrap();

    let commands = vec![serve_cmd(), stop_cmd(), php_list_cmd()];

    let app = App::new("rymfony")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
        .subcommands(commands);

    let matches = app.get_matches();

    let subcommand_name = matches.subcommand_name();

    match subcommand_name {
        Some("serve") => {
            serve(matches.subcommand_matches("serve").unwrap());
        }
        Some("stop") => {
            stop();
        }
        Some("php:list") => {
            php_list();
        }
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
