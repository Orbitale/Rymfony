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
