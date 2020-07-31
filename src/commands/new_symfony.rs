use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

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

    let mut path = PathBuf::from(&directory);

    if !path.is_absolute() {
        path = PathBuf::from(env::current_dir().unwrap())
            .join(&directory);
    }

    if path.is_dir() {
        error!("Directory {} already exists. Please choose another directory to create your project.", &path.to_str().unwrap());

        return;
    }

    info!("Using directory {}", &path.to_str().unwrap());

    let composer_path = which::which("composer").unwrap();

    info!("Composer detected at {}", &composer_path.to_str().unwrap());

    let mut command = Command::new(composer_path.to_str().unwrap());
    let package = String::from("symfony/skeleton");

    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("create-project")
        .arg(&package)
        .arg(path.to_str().unwrap())
    ;

    match command.output() {
        Ok(_) => {
            info!("Done!");
        },
        Err(e) => {
            error!("Could not create project");
            error!("{}", e);
        }
    };
}
