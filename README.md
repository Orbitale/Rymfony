Rymfony
=======

Rymfony is a work-in-progress command-line tool to mimic the behavior of the Symfony CLI binary.

## Install

### Download latest builds

The binary is built on every push to the `main` branch, if the binary can be built of course, and pushed to a dedicated server.

This allows you to test the latest version right away!

Here are the links to download them:

* Ubuntu: [rymfony.ubuntu-latest](https://files.pierstoval.com/rymfony/rymfony.ubuntu-latest?download)
* Windows: [rymfony.windows-latest.exe](https://files.pierstoval.com/rymfony/rymfony.windows-latest.exe?download)
* MacOS: [rymfony.macOS-latest](https://files.pierstoval.com/rymfony/rymfony.macOS-latest?download)

If you need more architectures and OSes, feel free to check the [build.yaml](./.github/workflows/build.yaml) Github Action and contribute for more!

### Manual install

Make sure you have installed the Rust language and its package manager Cargo on your machine, and run `cargo build --release`.

The binary will be stored in `./target/release/rymfony`, and you can use it.

## Usage

Run `rymfony help` to see the list of available commands:

```
$ rymfony
rymfony 0.1
Alex Rock <alex@orbitale.io>
To be determined

USAGE:
    rymfony [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help        Prints this message or the help of the given subcommand(s)
    php:list    List all available PHP executables
    serve       Runs an HTTP server
    stop        Stops a potentially running HTTP server
```

**Note:** For any command, you can use the `-h|--help` flag to display its details too. If you are familiar with the Symfony console component it is very much similar.

## Commands

### `rymfony serve`

⚠️ For now, this command launches the server, but the PHP server has to be implemented (check the roadmap for more information).

This command allows you to run a web server, in foreground or background, and you can customize the socket to listen to.

```
$ rymfony serve --help
rymfony-serve 
Runs an HTTP server

USAGE:
    rymfony serve [FLAGS] [OPTIONS]

FLAGS:
    -d, --daemon     Run the server in the background
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --listen <listen>    The TCP socket to listen to, usually an IP with a Port [default: 127.0.0.1:8000]
```

### `rymfony stop`

If a server is actually in a background running for the current project, it will be stopped.

Note that this is checked via a `.pid` file, containing the PID of the running server.

### `rymfony php:list`

This will list all existing `php` binaries in your environment.

It will actually search in the `PATH` directories for any binary that matches some patterns like these:

* On Windows:
  * `php.exe`
  * `phpX.Y.exe`
  * `php-cgi.exe`
  * `phpX.Y-cgi.exe`
  * `php-cgiX.Y.exe`
* On other platforms:
  * `php`
  * `phpX.Y`
  * `php-fpm`
  * `phpX.Y-fpm`
  * `php-fpmX.Y`

Here is for example the output on my Ubuntu 20.04 machine:

```
$ rymfony php:list
┌──────────────────────┐
| Binary path          |
├──────────────────────┤
| /usr/sbin/php-fpm7.4 |
| /usr/bin/php         |
| /usr/bin/php7.4      |
| /sbin/php-fpm7.4     |
| /bin/php             |
| /bin/php7.4          |
└──────────────────────┘
```

## Roadmap

**If you want to contribute to any of these points, feel free to do it!**

- To do:
    - [ ] Publish releases of the binary as artifacts by using Github Actions.
    - [ ] Make sure the server process is **totally** detached from the terminal in Windows. There are some issues about this, and it needs more investigation. Check [this blog post section](https://www.orbitale.io/2020/06/25/being-a-php-developer-on-windows-10-cool-snippets.html#3-symfony-binary-the-http-server) for more information.
    - [ ] Once a "way to start PHP" is found (either via CGI on Windows, FPM on Linux, or PHP's native server for other cases), make sure we can start a background PHP process.
    - [ ] Transform the standard web server into an HTTP proxy to PHP.
    - [ ] When the server is stopped (via Ctrl+C or via a panic), make sure PHP is stopped too.
    - [ ] Allow listing running servers globally, without necessarily using a `.pid` file.
    - [ ] Allow stopping a server globally, without necessarily using a `.pid` file.
    - [ ] When searching for PHP binaries, be able to flag their type (native, cgi, fpm) and their version.
    - [ ] Search for PHP binaries elsewhere than in `PATH`, such as with Homebrew or phpenv. This will need many checks about the "standard locations" where PHP can be found.
    - [ ] Allow passing environment variables to PHP via an `-e|--env` option.
    - [ ] Allow passing a custom option to specify which method the user wants to use to start PHP (like `--use-fpm` or `--use-native`, something like that).
    - [ ] (utopia) Support setups that have multiple PHP versions installed (such as on Ubuntu/Debian with deb-sury's repo, or with Homebrew on OSX), and allow customizing the version.
    - [ ] (utopia) Detect whether the project uses Docker Compose
    - [ ] (utopia) Be able to dynamically create environment variables for some common use-cases (database, redis, rabbitmq, mailcatcher).
- Done:
    - [x] Make sure we can run a web server using Hyper and Tokio.
    - [x] Put the web-server execution in a separate `serve.rs` file.
    - [x] Split commands in a `src/commands/` subdirectory for an easier code organization.
    - [x] Separate the "App" command definition (using the `clap` crate) and put it in each command's own dir (best example is how the `serve` command is defined).
    - [x] Make sure the web server's IP and port can be customized through a `--listen ip:port` option. 
    - [X] Execute the server in the background.
    - [X] Add a `stop` command.
    - [x] Create a tool to discover the `php-cgi` binary (I'm developing on Windows and it is therefore easier). 
    - [x] Create a tool to discover the `php-fpm` binary if on Linux.
    - [x] Create a tool to discover the `php` binary if none of the two above are detected.
    - [x] Publish nightly builds of the binary as artifacts by using Github Actions.
