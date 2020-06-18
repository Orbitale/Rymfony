Rymfony
=======

Rymfony is a work-in-progress command-line tool to mimic the behavior of the Symfony CLI binary.

The project is **very young**. Remember that it **me learning Rust**, therefore there might be tons of mistakes, and the roadmap will keep evolving, as well as this documentation.

It is open-source for showcase and code reviews, and maybe to help other Rust newcomers understand what can be done with this cool language.

## Install

Make sure you have installed the Rust language and its package manager Cargo on your machine.

## Usage

Run `cargo run serve` to run the web server.

## Roadmap

There are **a lot** of things I would like to do, and I think it will drastically help me understand how Rust works, therefore I will check these checkboxes every time something is achieved. And maybe I will add more in the future too. 

- [x] Make sure we can run a web server using Hyper and Tokio.
- [x] Put the web-server execution in a separate `serve.rs` file.
- [x] Split commands in a `src/commands/` subdirectory for an easier code organization.
- [x] Separate the "App" command definition (using the `clap` crate) and put it in each command's own dir (best example is how the `serve` command is defined).
- [x] Make sure the web server's IP and port can be customized through a `--listen ip:port` option. 
- [ ] Execute the server in the background.
- [ ] Create a tool to discover the `php-cgi` binary (I'm developing on Windows and it is therefore easier). 
- [ ] Create a tool to discover the `php-fpm` binary if on Linux.
- [ ] Create a tool to discover the `php` binary if none of the two above are detected.
- [ ] Once a "way to start PHP" is found (either via CGI on Windows, FPM on Linux, or PHP's native server for other cases), make sure we can start a background PHP process.
- [ ] When the server is stopped (via Ctrl+C or via a panic), make sure PHP is stopped too.
- [ ] Transform the standard web server into an HTTP proxy to PHP.
- [ ] Allow passing environment variables to PHP via an `-e|--env` option.
- [ ] Allow passing a custom option to specify which method the user wants to use to start PHP (like `--use-fpm` or `--use-native`, something like that).
- [ ] Support setups that have multiple PHP versions installed (such as on Ubuntu/Debian with deb-sury's repo, or with Homebrew on OSX), and allow customizing the version.
- [ ] (utopia) Detect whether the project uses Docker Compose
- [ ] (utopia) Be able to dynamically create environment variables for some common use-cases (database, redis, rabbitmq, mailcatcher).
