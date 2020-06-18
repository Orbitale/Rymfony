mod commands {
    pub(crate) mod serve;
}

use clap::App;

use crate::commands::serve::command_config as serve_cmd;
use crate::commands::serve::serve;

fn main() {
    let commands = vec![
        serve_cmd()
    ];

    let app = App::new("rymfony")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
        .subcommands(commands)
    ;

    let matches = app.get_matches();

    let subcommand_name = matches.subcommand_name();

    println!("Executing command: {}", subcommand_name.unwrap());

    match subcommand_name {
        Some("serve") | _ => {
            serve(matches.subcommand_matches("serve").unwrap());
        }
    }
    ;
}
