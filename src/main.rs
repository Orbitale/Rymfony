mod commands {
    pub(crate) mod serve;
}

mod utils {
    pub(crate) mod current_process_name;
}

use crate::commands::serve::command_config as serve_cmd;
use crate::commands::serve::serve;
use clap::App;
use clap::ArgMatches;
use std::env;
use std::process::Command;
use std::process::Stdio;
use utils::current_process_name;

fn main() {
    let commands = vec![serve_cmd()];

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
        _ => {
            // If no subcommand is specified,
            // re-run the program with "--help"
            let mut subprocess = Command::new(current_process_name::get().as_str())
                .arg("--help")
                .spawn()
                .expect("Failed to start sub process");
            subprocess.wait();
        }
    };
}
