# v0.4.4

* Fix a bug in the arguments of `rymfony serve --daemon`

# v0.4.3

* Update all Rust dependencies
* In the Rust-deps update, Clap was updated to v4, so big changes were made in how CLI commands are implemented. The setup is now a bit more abstract, making it a bit easier to maintain (despite abstraction). All base command executors now return instances of `ExitCode`, to be consistent with CLI. (ultimate goal would be to get rid of `panic!()` and just handle everything with `Result` or `Option` objects)
* Update Caddy to v2.6.2
* Added `--host` option to `rymfony serve`
* When using `-vvv` (debug), the `debug` option is also set in Caddy for logging
* Enforce usage of the HTTP/HTTPS protocol in the `Caddyfile`
* Added a `Dockerfile` (⚠ work in progress, only experimental for now)
* Reformat all code with `caddy fmt` and create a `.rustfmt.toml` file for that 

# v0.4.2

* Don't build on MacOS anymore, because Github Action is super slow with Mac (see [this run](https://github.com/Orbitale/Rymfony/actions/runs/3000081339))

# v0.4.1

* Updated all dependencies
* Enforced php 8.1 on the test setup
* Update caddy from 2.5.1 to 2.5.2
* Fix tests behaving differently on different CI builds

# v0.4.0

* Add a logging system that you can use by running `rymfony log` in your project root directory. Will tail log files from your running `rymfony` server, even when server is not started.
* Enhanched "healthcheck" so that both Caddy HTTP server and PHP-FPM/PHP-CGI servers can be restarted automatically when they fail. Very useful on Windows when PHP-CGI fails for concurrency/overloading reasons.
* Use latest versions of dependencies.
* Latest version of the `clap` package enhances the output of all command-line documentation, like `rymfony help`. (especially adds colors to your terminal!)
* Add functional tests to check for cross-OS consistency in how HTTP and PHP are handled.
* Enhance how Caddy HTTP server permissions are checked: before, `setcap` was executed to allow Caddy to bind port 80 when running as non-root.<br>This restriction happened because Linux prevents non-root to bind ports below 1024, for security reasons.<br>Now, `setcap` is run in a non-blocking way when a TTY is detected, so that Caddy can fail gracefully, but still blocks and wait for user input when a console/shell/terminal output is detected (like when in login/interactive mode).<br>Also, the `Caddyfile` template was modified to ensure that port 80 is no longer listened (this is linked to the _"Redirect HTTP to HTTPS"_ feature from Caddy).

# v0.3.1

* Add a "healthcheck" system to restart php-fpm/php-cgi when it fails.
* Disable "native php" server support (it's not customizable, slow, not cgi, so we don't recommend it anyway).
* Tidy the different config/project paths properly, and enhance some error messages for better debugging.
* Tidy the PIDs management so that they're easier to discover and manage.

# v0.3.0

* Embed Caddy server into Rymfony, instead of using Warp (by @Pierstoval in https://github.com/Orbitale/Rymfony/pull/85)

# v0.2.3

* Allow to run php-fpm (and by extension Rymfony itself) as root by @Shine-neko in #86

# v0.2.2

* Don't display Rymfony version in "SERVER_SOFTWARE" header
* Fis compatibility with MacOS Big Sur

# v0.2.1

* PHP-FPM logs are redirected to a file in the Rymfony project directory (located in the `$HOME/.rymfony/{hash}/log/` directory) instead of being redirected to `stderr`.
* Send the right `SERVER_SOFTWARE` header through FastCGI.

# v0.2.0

* Update wording, help messages and error messages
* Remove Rust 2021 deprecations
* Update all Cargo dependencies
* Update FastCGI client dependency: now it's async! 🎉

# v0.1.1

* Added a better way to detect `systemd` (See [source](https://www.freedesktop.org/software/systemd/man/sd_booted.html) if you want to know how to do it).
* Removed `is_wsl` dependency.
* Updated all other dependencies.

# v0.1.0

Initial release
