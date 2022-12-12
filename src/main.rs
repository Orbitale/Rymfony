#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate env_logger;
extern crate pretty_env_logger;
extern crate regex;
extern crate tokio;

mod command_handling;
mod logging;

mod config {
    pub(crate) mod config;
    pub(crate) mod paths;
}

mod commands {
    pub(crate) mod logs;
    pub(crate) mod new_symfony;
    pub(crate) mod php_list;
    pub(crate) mod serve;
    pub(crate) mod stop;
}

mod utils {
    pub(crate) mod current_process_name;
    pub(crate) mod network;
    pub(crate) mod project_directory;
    pub(crate) mod stop_process;
}

mod php {
    pub(crate) mod binaries;
    pub(crate) mod php_server;
    pub(crate) mod server_cgi;
    pub(crate) mod server_fpm;
    pub(crate) mod structs;
}

mod http {
    pub(crate) mod caddy;
    pub(crate) mod proxy_server;
}

use crate::command_handling::CommandList;
use clap::Arg;
use clap::ArgAction;
use clap::ColorChoice;
use clap::Command as ClapCommand;
use dirs::home_dir;
use std::fs;
use std::process::Command;
use std::process::ExitCode;

fn application_commands() -> CommandList {
    CommandList {
        commands: vec![
            Box::new(commands::logs::get_command()),
            Box::new(commands::php_list::get_command()),
            Box::new(commands::serve::get_command()),
            Box::new(commands::stop::get_command()),
            Box::new(commands::new_symfony::get_command()),
        ],
    }
}

const APPLICATION_NAME: &str = "rymfony";
const APP_VERSION_METADATA: &str = include_str!("../.version");

fn main() -> ExitCode {
    startup_actions();

    let application_commands = application_commands();

    let subcommands = application_commands.subcommands().into_iter();

    let application = get_application().subcommands(subcommands);

    let arg_matches = application.get_matches();

    let verbosity_level: &u8 = arg_matches.get_one::<u8>("verbose").unwrap_or(&0);
    let is_quiet = arg_matches.get_flag("quiet");

    logging::set_verbosity_value(*verbosity_level, is_quiet);

    let subcommand_name = arg_matches.subcommand_name();
    let args = if let Some(subcommand_name) = subcommand_name {
        arg_matches.subcommand_matches(subcommand_name)
    } else {
        None
    };

    if let Some(subcommand_name) = subcommand_name {
        for command in application_commands.commands.iter() {
            if command.command_definition.get_name() == subcommand_name {
                return (command.executor)(args.unwrap());
            }
        }
    }

    default_command().unwrap_or(ExitCode::FAILURE)
}

fn startup_actions() {
    let home_dir = home_dir().unwrap();
    if home_dir.to_str().unwrap() != "" {
        fs::create_dir_all(home_dir.join(".rymfony")).unwrap();
    }
}

fn get_application() -> ClapCommand {
    ClapCommand::new(APPLICATION_NAME)
        .version(APP_VERSION_METADATA.trim())
        .author("Alex \"Pierstoval\" Rock <alex@orbitale.io>")
        .color(ColorChoice::Always)
        .about(
            "
A command-line tool to spawn a PHP server behind an HTTP FastCGI proxy,
inspired by Symfony CLI, but Open Source.

https://github.com/Orbitale/Rymfony
",
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .global(true)
                .action(ArgAction::Count)
                .help("Set the verbosity level. -v for debug, -vv for trace, -vvv to trace executed modules"),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .global(true)
                .num_args(0)
                .help("Do not display any output. Has precedence over -v|--verbose"),
        )
}

fn default_command() -> Option<ExitCode> {
    let process_args: Vec<String> = std::env::args().collect();
    let current_process_name = process_args[0].as_str().to_owned();

    // If no subcommand is specified,
    // re-run the program with "--help"
    let mut subprocess = Command::new(&current_process_name).arg("--help").spawn().ok()?;

    let child = subprocess.wait().ok()?;

    let status = child.code();

    match status {
        Some(code) => Some(ExitCode::from(code as u8)),
        None => Some(ExitCode::FAILURE),
    }
}

#[cfg(test)]
mod main_tests {
    use super::*;

    #[test]
    fn verify_cli() {
        get_application().debug_assert();
    }
}
