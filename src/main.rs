mod commands {
    pub(crate) mod serve;
}

use clap::App;
use clap::Arg;

use crate::commands::serve::serve;

fn main() {
    let app = App::new("rustphp")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
        .subcommands(vec![
            App::new("serve")
                .about("Runs an HTTP server")
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .help("The HTTP port to listen to")
                )
        ])
    ;

    let matches = app.get_matches();

    match matches.subcommand_name() {
        Some("serve")  => {
            println!("Executed command: {}", matches.subcommand_name().unwrap());
            serve(matches);
        },
        _ => {
            println!("Executed no command. Execute \"serve\" by default");
            serve(matches);
        },
    }
    ;
}
