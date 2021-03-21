[![Gitpod Ready-to-Code](https://img.shields.io/badge/Gitpod-ready--to--code-blue?logo=gitpod)](https://gitpod.io/#https://github.com/Orbitale/Rymfony)

Rymfony
=======

Rymfony is a command-line tool to mimic the behavior of the Symfony CLI binary.

## Install

To install Rymfony, [download your version from the latest release on the Releases page](https://github.com/Orbitale/Rymfony/releases).

### Download latest dev builds

The binary is built on every push to the `main` branch, if the binary can be built of course, and pushed to [nightly.link](https://nightly.link).

This allows you to test the latest version right away!

Here are the links to download them:

| [![nightly.ubuntu-latest](https://img.shields.io/badge/Ubuntu%20nightly-download-brightgreen)](https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.ubuntu.zip) |
|:--|
| [![nightly.windows-latest](https://img.shields.io/badge/Windows%20nightly-download-brightgreen)](https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.windows.zip) |
| [![nightly.macOS-latest](https://img.shields.io/badge/MacOS%20nightly-download-brightgreen)](https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.macOS.zip) |
| [![All nightly releases](https://img.shields.io/badge/All%20releases-download-brightgreen)](https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main) |

Unzip the file and put the `rymfony` executable file in your `PATH`, and you're set!

If you need more architectures and OSes, feel free to check the [Build.yaml](.github/workflows/Build.yaml) Github Action and contribute for more!

#### Download on Linux

```
curl -sSL https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.ubuntu.zip -o rymfony.zip && unzip rymfony.zip && sudo mv rymfony /usr/bin/rymfony && sudo chmod a+x /usr/bin/rymfony && rm rymfony.zip
```

#### Download on Windows

With `cmd` (`powershell` must be accessible):
```
powershell Invoke-WebRequest https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.windows.zip -OutFile rymfony.zip && powershell Expand-Archive -Force rymfony.zip . && rm rymfony.zip
```

With Powershell directly:
```
Invoke-WebRequest https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.windows.zip -OutFile rymfony.zip && Expand-Archive -Force rymfony.zip . && rm rymfony.zip
```

Then, add the `rymfony.exe` executable somewhere in your PATH.

#### Download on MacOS

```
curl -sSL https://nightly.link/Orbitale/Rymfony/workflows/Build.yaml/main/rymfony.macOS.zip -o rymfony.zip && unzip rymfony.zip && sudo mv rymfony /usr/local/bin/rymfony && sudo chmod a+x /usr/local/bin/rymfony && rm rymfony.zip
```

### Manual build

* First, install Rust following the [Official guide](https://www.rust-lang.org/tools/install).
* Clone the repository on your machine with this command:<br>
  `git clone git@github.com:Orbitale/Rymfony.git`.
* Then, run `cargo build --release`.
* Done!<br>
  The binary will be stored in `./target/release/rymfony` (with `.exe` extension on Windows), you can use it directly!

## Usage

Run `rymfony help` to see the list of available commands:

```
$ rymfony
rymfony 0.1.0-dev
Alex Rock <alex@orbitale.io>

A command-line tool to spawn a PHP server behind an HTTP FastCGI proxy,
inspired by Symfony CLI, but open-source.

https://github.com/Orbitale/Rymfony

USAGE:
    rymfony [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Do not display any output. Has precedence over -v|--verbose
    -V, --version    Prints version information
    -v, --verbose    Set the verbosity level. -v for debug, -vv for trace, -vvv to trace executed modules

SUBCOMMANDS:
    help                   Prints this message or the help of the given subcommand(s)
    new:symfony            Create a new Symfony project
    php:list               List all available PHP executables.
    server:ca:install      Create and install a local Certificate Authority for serving HTTPS
    server:ca:uninstall    Uninstall the local Certificate Authority
    server:start           Runs an HTTP server
    stop                   Stops a potentially running HTTP server
```

**Note:** For any command, you can use the `-h|--help` flag to display its details too. If you are familiar with the Symfony console component it is very much similar.

## Commands

### `rymfony serve` (or `server:start`)

This command allows you to run a web server, in foreground or background, and you can customize the port to listen to.

```
$ rymfony serve --help
rymfony-server:start
Runs an HTTP server

USAGE:
    rymfony server:start [FLAGS] [OPTIONS]

FLAGS:
        --allow-http              Do not redirect HTTP request to HTTPS
    -d, --daemon                  Run the server in the background
    -s, --expose-server-header    Add server header into all response
    -h, --help                    Prints help information
        --no-tls                  Disable TLS. Use HTTP only.
    -V, --version                 Prints version information

OPTIONS:
        --document-root <document-root>    Project's document root
        --passthru <passthru>              The PHP script all requests will be passed to
        --port <port>                      The TCP port to listen to [default: 8000]
```

### `rymfony stop`

If a server is running in the background running for the current project, it will be stopped.

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
  * `php-cgi`
  * `phpX.Y-fpm`
  * `phpX.Y-cgi`
  * `php-fpmX.Y`
  * `php-cgiX.Y`

More locations for standard PHP installations that are searched can be found in [binaries.rs](src/php/binaries.rs).

> ℹNote: if your PHP binary is not detected, please **open an issue** so we can add support for it!

Here is the output from an Ubuntu 20.04 machine:

```
$ rymfony php:list
┌─────────┬─────────────────┬──────────────────────┬─────────┬────────┐
| Version | PHP CLI         | PHP FPM              | PHP CGI | System |
├─────────┼─────────────────┼──────────────────────┼─────────┼────────┤
| 7.4.11  | /usr/bin/php7.4 | /usr/sbin/php-fpm7.4 |         | *      |
└─────────┴─────────────────┴──────────────────────┴─────────┴────────┘
```

Windows 10:

```
> rymfony php:list
┌─────────┬──────────────────────┬─────────┬──────────────────────────┬────────┐
| Version | PHP CLI              | PHP FPM | PHP CGI                  | System |
├─────────┼──────────────────────┼─────────┼──────────────────────────┼────────┤
| 7.4.2   | E:\dev\php74\php.exe |         | E:\dev\php74\php-cgi.exe | *      |
└─────────┴──────────────────────┴─────────┴──────────────────────────┴────────┘
```

macOS Catalina (using Homebrew): 

```
$ rymfony php:list
┌─────────┬───────────────────────────────────────────────┬──────────────────────────────────────────┬───────────────────────────────────────────────────┬────────┐
| Version | PHP CLI                                       | PHP FPM                                  | PHP CGI                                           | System |
├─────────┼───────────────────────────────────────────────┼──────────────────────────────────────────┼───────────────────────────────────────────────────┼────────┤
| 5.5.5   | /usr/local/php5-5.5.5-20131020-222726/bin/php |                                          | /usr/local/php5-5.5.5-20131020-222726/bin/php-cgi |        |
| 7.3.11  | /usr/bin/php                                  | /usr/sbin/php-fpm                        |                                                   |        |
| 7.3.21  | /usr/local/Cellar/php@7.3/7.3.21/bin/php      |                                          | /usr/local/Cellar/php@7.3/7.3.21/bin/php-cgi      |        |
| 7.4.9   | /usr/local/Cellar/php/7.4.9/bin/php           | /usr/local/Cellar/php/7.4.9/sbin/php-fpm | /usr/local/Cellar/php/7.4.9/bin/php-cgi           | *      |
└─────────┴───────────────────────────────────────────────┴──────────────────────────────────────────┴───────────────────────────────────────────────────┴────────┘
```

*ℹ Note:* To search for PHP executables in a custom folder, you can define the `RYMFONY_PATH` environment variable.

On *nix systems, you must use the colon `:` path separator.<br>
On Windows systeme use the semi-colon `;` path separator.

```
# *nix
$ export RYMFONY_PATH=/home/php/php-dev:/var/local/php-dev

# Windows
> set RYMFONY_PATH=c:\php7.4;d:\php8.0;d:\php7.3
```

## Roadmap

**If you want to contribute to any of these points, feel free to do it!**

- 🟩 : Done
- 🟨 : In progress (a PR should be linked)
- 🟥 : Planned, but not started yet

To do (order of priority, done first):

- Commands and command-line options
    - 🟩 Add a `stop` command.
    - 🟥 Create `open:local` command.
    - 🟥 Create `server:logs` command (needs #81 first).
    - 🟥 Create `server:list` command.
    - 🟥 Create `server:status` command.
    - 🟥 Create `config` command, to display project's config, and maybe change it.
    - 🟥 Create `php` command (should use the configured PHP version).
    - 🟥 Create `console` command for Symfony (should use the configured PHP version, and be compatible with SF 2+).
    - 🟥 Create `artisan` command for Laravel (should use the configured PHP version).
    - 🟥 Propagate global app arguments so they can be used in subcommands (like `rymfony serve -v`, because if you want verbosity today, you need to pass the option before the subcommand, like this: `rymfony -v serve`). Maybe this can be in the `clap` crate itself (the CLI app package used by Rymfony).
- Releases
    - 🟩 Publish nightly builds of the binary as artifacts by using Github Actions.
    - 🟩 Add support for verbosity levels in output logging, like `-v`, `-vv`, `-vvv` and `-q`.
    - 🟩 Add version hash to nightly builds.
    - 🟩 Publish releases of the binary as artifacts by using Github Actions. For now, only "nightly" builds are released.
- HTTP server
    - 🟩 Make sure we can run a web server using Hyper and Tokio.
    - 🟩 Put the web-server execution in a separate `serve.rs` file.
    - 🟩 Execute the server in the background.
    - 🟩 Make sure the web server's IP and port can be customized through a `--listen ip:port` option. 
    - 🟩 Once a "way to start PHP" is found (either via CGI on Windows, FPM on Linux, or PHP's native server for other cases), make sure we can start a background PHP process.
    - 🟩 Transform the standard web server into an HTTP proxy to PHP using a FastCGI client
    - 🟥 #81 Tail logs to a file when server is run in the background
    - 🟥 Make sure the server process is **totally** detached from the terminal in Windows. There are some issues about this, and it needs more investigation. Check [this blog post section](https://www.orbitale.io/2020/06/25/being-a-php-developer-on-windows-10-cool-snippets.html#3-symfony-binary-the-http-server) for more information.
    - 🟥 When the server is stopped (via Ctrl+C or via a panic), make sure PHP is stopped too.
    - 🟥 Allow listing running servers globally, without necessarily using a `.pid` file.
    - 🟥 Allow stopping a server globally, without necessarily using a `.pid` file.
    - 🟥 (possibly hard work) Find a way to force the entire request-response workflow to be streamed instead of buffered (will make better performances and memory usage)
- PHP server
    - 🟩 Create a tool to discover the `php` binary if none of the two above are detected.
    - 🟩 Create a tool to discover the `php-fpm` binary if on Linux.
    - 🟩 Create a tool to discover the `php-cgi` binary (I'm developing on Windows and it is therefore easier).
    - 🟥 Properly search for PHP binaries in the current machine
        - 🟩 When searching for PHP binaries, be able to flag their type (native, cgi, fpm) and their version. 
        - 🟩 Implement a way to retrieve the current PHP version based on the "System" PHP script
        - 🟥 Search for PHP binaries elsewhere than in `PATH`, such as with Homebrew or phpenv. This will need many checks about the "standard locations" where PHP can be found.
            - 🟩 Search in `/usr/bin` for most Ubuntu defaults
            - 🟩 Search in `/usr/local/Cellar` for most Homebrew defaults on Mac
            - 💡 Please [suggest](https://github.com/Pierstoval/rymfony/issues/new) more places where PHP could be present!
        - 🟩 Flag the current path-based `php` script to check its version and mark it as "System" (just like in Symfony CLI)
        - 🟩 Store a list of all PHP binaries in `~/.rymfony/php_versions.json`
        - 🟩 Deserialize the `php-versions.json` config file if it exists when using `binaries::all()` or `binaries::current()` to make the command faster
        - 🟩 Add an option to the `php:list` command such as `--refresh` that will make another lookup and save the `~/.rymfony/php-versions.json` file again 😄
        - 🟩 Implement a way to retrieve the current PHP version based on a local `.php-version` file
    - 🟥 Allow passing environment variables to PHP via an `-e|--env` option.
    - 🟥 Allow passing a custom option to specify which method the user wants to use to start PHP (like `--use-fpm` or `--use-native`, something like that).
    - 🟥 (utopia) Support setups that have multiple PHP versions installed (such as on Ubuntu/Debian with deb-sury's repo, or with Homebrew on OSX), and allow customizing the version.
    - PHP Server
        - 🟩 Don't rewrite the `fpm-conf.ini` configuration file each time a server is launched.
        - 🟩 Find a way to differenciate servers configurations, in case multiple servers are started: Done by creating a specific PHP-FPM configuration for each project 🙂
- Going way further
    - 🟥 (utopia) Detect whether the project uses Docker Compose
    - 🟥 (utopia) Be able to dynamically create environment variables for some common use-cases (database, redis, rabbitmq, mailcatcher).
