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
            Arg::with_name("directory")
                .index(1)
                .help("The directory in which the project will be created"),
        )
        .arg(
            Arg::with_name("full")
                .long("full")
                .help("Use the symfony/website-skeleton instead of the default one")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("no-git")
                .long("no-git")
                .help("Do not initialize the project with git"),
        )
}

pub(crate) fn new_symfony(args: &ArgMatches) {
    let mut directory = args.value_of("directory").unwrap_or("").to_string();
    let full = args.is_present("full");
    let initialize_git = !args.is_present("no_git");

    if directory == "" {
        let path = env::current_dir().unwrap().join("symfony");

        directory = path.to_str().unwrap().to_string();
    }

    let mut path = PathBuf::from(&directory);

    if !path.is_absolute() {
        path = PathBuf::from(env::current_dir().unwrap()).join(&directory);
    }

    if path.is_dir() {
        error!(
            "Directory {} already exists. Please choose another directory to create your project.",
            &path.to_str().unwrap()
        );

        return;
    }

    info!("Using directory {}", &path.to_str().unwrap());

    let composer_path = which::which("composer").unwrap();

    info!("Composer detected at {}", &composer_path.to_str().unwrap());

    let mut command = Command::new(composer_path.to_str().unwrap());

    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .arg("create-project")
        .arg(if full {
            "symfony/website-skeleton"
        } else {
            "symfony/skeleton"
        })
        .arg(path.to_str().unwrap());

    match command.output() {
        Ok(_) => (),
        Err(e) => {
            error!("Could not create project");
            error!("{}", e);
        }
    };

    if initialize_git {
        let git_path = which::which("git").unwrap();

        let mut command = Command::new(git_path.to_str().unwrap());

        command
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .arg("init")
            .arg(path.to_str().unwrap());

        match command.output() {
            Ok(_) => (),
            Err(e) => {
                error!("Could not initialize git");
                error!("{}", e);
            }
        };
    }
}
