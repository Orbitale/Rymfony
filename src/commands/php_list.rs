use crate::command_handling::CommandHandler;
use clap::arg;
use clap::ArgMatches;
use clap::Command as ClapCommand;
use prettytable::format;
use prettytable::Table;
use std::process::ExitCode;

use crate::config::config::clear_binaries_list;
use crate::config::config::save_binaries_to_config;
use crate::php;

pub(crate) fn get_command() -> CommandHandler {
    CommandHandler::new(
        ClapCommand::new("php:list")
        .about("List all available PHP executables.")
        .after_help("
If you have PHP installed in a custom folder, you can use the RYMFONY_PATH environment variable before executing the command.

Example:

$ RYMFONY_PATH=\"/var/php80/bin\" rymfony php:list --refresh
")
        .arg(arg!(-r --refresh "Refresh the PHP list cache"))
        ,
        Box::new(execute),
    )
}

pub(crate) fn execute(args: &ArgMatches) -> ExitCode {
    if args.get_flag("refresh") {
        match clear_binaries_list() {
            Ok(_) => info!("Binaries cache successfully cleared!"),
            Err(e) => {
                error!("Could not clear binaries cache: {}", e);

                return ExitCode::from(1);
            },
        }
    }

    let binaries = php::binaries::all();

    if binaries.len() == 0 {
        error!("No PHP installation found. To provide your specific PHP installation path, you can use the RYMFONY_PATH environment variable before running \"rymfony php:list --refresh\".");

        return ExitCode::from(1);
    }

    save_binaries_to_config(&binaries);

    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('│')
        .borders('│')
        .separators(&[format::LinePosition::Top], format::LineSeparator::new('─', '┬', '┌', '┐'))
        .separators(&[format::LinePosition::Bottom], format::LineSeparator::new('─', '┴', '└', '┘'))
        .separators(&[format::LinePosition::Title], format::LineSeparator::new('─', '┼', '├', '┤'))
        .padding(1, 1)
        .build();

    table.set_format(format);
    table.set_titles(row!["Version", "PHP CLI", "PHP FPM", "PHP CGI", "System"]);

    let mut ordered_binaries: Vec<_> = binaries.into_iter().collect();
    ordered_binaries.sort_by(|x, y| x.0.version().cmp(y.0.version()));

    for (php_version, php_binary) in ordered_binaries {
        let system = if php_binary.system() { "*" } else { "" };
        table.add_row(row![
            php_version.version(),
            php_binary.cli(),
            php_binary.fpm(),
            php_binary.cgi(),
            system
        ]);
    }

    table.printstd();

    ExitCode::from(0)
}
