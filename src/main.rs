mod commands {
    pub(crate) mod serve;
    pub(crate) mod stop;
}

mod utils {
    pub(crate) mod current_process_name;
}

use crate::commands::serve::command_config as serve_cmd;
use crate::commands::serve::serve;
use crate::commands::stop::command_config as stop_cmd;
use crate::commands::stop::stop;
use clap::App;
use std::process::Command;
use utils::current_process_name;

fn main() {
    let commands = vec![serve_cmd(), stop_cmd()];

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
        _ => {
            // If no subcommand is specified,
            // re-run the program with "--help"
            let mut subprocess = Command::new(current_process_name::get().as_str())
                .arg("--help")
                .spawn()
                .expect("Failed to start sub process");
            subprocess
                .wait()
                .expect("An error occured when trying to execute default command");
        }
    };
}
