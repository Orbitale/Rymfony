use clap::App;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use linemux::MuxedLines;
use crate::config::paths;
use colored::*;

pub(crate) fn command_config<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("logs")
        .alias("log")
        .alias("local:server:log") // For Symfony CLI compat
        .alias("server:log") // For Symfony CLI compat
        .about("Display server logs")
        .arg(
            Arg::with_name("channel")
                .index(1)
                .help("The optional logging channel you want to display"),
        )
        .arg(
            Arg::with_name("no-follow")
                .long("no-follow")
                .alias("no-tail")
                .help("Do no tail the logs")
        )
        .arg(
            Arg::with_name("lines")
                .short("n")
                .long("lines")
                .help("Number of lines to display at start")
                .takes_value(true)
                .default_value("0"),
        )
}

pub(crate) fn logs(args: &'_ ArgMatches) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    rt.block_on(async {
        let mut lines = MuxedLines::new().unwrap();

        lines.add_file(paths::get_rymfony_process_log_file()).await.unwrap();
        lines.add_file(paths::get_rymfony_process_err_file()).await.unwrap();

        lines.add_file(paths::get_http_process_stdout_file()).await.unwrap();
        lines.add_file(paths::get_http_process_stderr_file()).await.unwrap();

        lines.add_file(paths::get_http_server_log_file()).await.unwrap();
        lines.add_file(paths::get_http_vhost_log_file()).await.unwrap();

        lines.add_file(paths::get_php_process_log_file()).await.unwrap();
        lines.add_file(paths::get_php_process_err_file()).await.unwrap();

        lines.add_file(paths::get_php_server_log_file()).await.unwrap();
        lines.add_file(paths::get_php_server_error_file()).await.unwrap();

        while let Ok(Some(line)) = lines.next_line().await {
            let source = line.source().file_name().unwrap();
            println!("[{}] - {}", source.to_str().unwrap().green(), line.line());
        }
    });
}