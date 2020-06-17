mod commands {
    pub(crate) mod serve;
}

use clap::App;
use clap::Arg;

use crate::commands::serve::serve;

const DEFAULT_LISTEN: &str = "127.0.0.1:5000";

fn main() {
    let app = App::new("rymfony")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
        .subcommands(vec![
            App::new("serve")
                .about("Runs an HTTP server")
                .arg(
                    Arg::with_name("listen")
                        .short("l")
                        .long("listen")
                        .default_value(DEFAULT_LISTEN)
                        .help("The TCP socket to listen to, usually an IP with a Port")
                )
        ])
    ;

    let matches = app.get_matches();

    let listen = match matches.value_of("listen") {
        Some(t) => {
            println!("Something something {}", t);
            t
        }
        None => DEFAULT_LISTEN
    };

    match matches.subcommand_name() {
        Some("serve")  => {
            println!("Executed command: {}", matches.subcommand_name().unwrap());

            serve(listen);
        },
        _ => {
            println!("Executed no command. Execute \"serve\" by default");
            serve(listen);
        },
    }
    ;
}
