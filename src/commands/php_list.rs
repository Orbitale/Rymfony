use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use prettytable::format;
use prettytable::Table;

use crate::config::config::clear_binaries_list;
use crate::config::config::save_binaries_to_config;
use crate::php;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("php:list").about("List all available PHP executables.

If you have PHP installed in a custom folder, you can use the RYMFONY_PATH environment variable before executing the command.

Example:

$ RYMFONY_PATH=\"/var/php80/bin\" rymfony php:list --refresh
")
        .arg(
            Arg::with_name("refresh")
                .short("r")
                .long("refresh")
                .help("Refresh the PHP list cache"),
        )
}

pub(crate) fn php_list(args: &ArgMatches) {
    if args.is_present("refresh") {
        match clear_binaries_list() {
            Ok(_) => info!("Binaries cache successfully cleared!"),
            Err(e) => error!("Could not clear binaries cache: {}", e),
        }
    }

    let binaries = php::binaries::all();

    if binaries.len() == 0 {
        error!("No PHP installation found. To provide your specific PHP installation path, you can use the RYMFONY_PATH environment variable before running \"rymfony php:list --refresh\".");
        return;
    }

    save_binaries_to_config(&binaries);

    let mut table = Table::new();
    let format = format::FormatBuilder::new()
        .column_separator('│')
        .borders('│')
        .separators(
            &[format::LinePosition::Top],
            format::LineSeparator::new('─', '┬', '┌', '┐'),
        )
        .separators(
            &[format::LinePosition::Bottom],
            format::LineSeparator::new('─', '┴', '└', '┘'),
        )
        .separators(
            &[format::LinePosition::Title],
            format::LineSeparator::new('─', '┼', '├', '┤'),
        )
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
}
