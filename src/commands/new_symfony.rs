use clap::arg;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};
use std::process::Stdio;
use crate::command_handling::CommandHandler;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("new:symfony")
            .alias("new") // For Symfony CLI compat
            .about("Create a new Symfony project")
            .arg(arg!(<directory> "The directory in which the project will be created"))
            .arg(arg!(--full "Use the symfony/website-skeleton instead of the default one"))
            .arg(arg!(--"no-git" "Do not initialize the project with git"))
        ,
        Box::new(execute),
    )
}

pub(crate) fn execute(args: &ArgMatches) -> ExitCode {
    let mut directory = args.get_one::<String>("directory").map(|s| s.as_str()).unwrap_or("").to_string();
    let full = args.get_flag("full");
    let initialize_git = !args.get_flag("no-git");

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

        return ExitCode::from(1);
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

            return ExitCode::from(1);
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

                return ExitCode::from(1);
            }
        };
    }

    ExitCode::from(0)
}
