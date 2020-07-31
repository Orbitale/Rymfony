use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use std::env;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("new:symfony")
        .alias("new")
        .about("Create a new Symfony project")
        .arg(
            Arg::with_name("dit")
                .index(1)
                .help("The directory in which the project will be created"),
        )
}

pub(crate) fn new_symfony(args: &ArgMatches) {
    let mut directory = args.value_of("dit").unwrap_or("").to_string();

    if directory == "" {
        let path = env::current_dir().unwrap()
            .join("symfony");

        directory = path.to_str().unwrap().to_string();
    }

    info!("Creating a new project in \"{}\"", &directory);
}
