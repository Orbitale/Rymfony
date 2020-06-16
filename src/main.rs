mod commands {
    pub(crate) mod serve;
}

use clap::App;

use crate::commands::serve::cmd as serve_cmd;

fn main() {
    let app = App::new("rustphp")
        .version("0.1")
        .author("Alex Rock <alex@orbitale.io>")
        .about("To be determined")
    ;

    let commands = vec![
        serve_cmd(),
    ];

    for subcmd in commands {
        app.subcommand(subcmd); // Doesn't work -_- need to investigate
    }

    let matches = app.get_matches();
    // TODO
}
