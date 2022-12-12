use crate::APPLICATION_NAME;
use log::Level;
use pretty_env_logger::env_logger::fmt::Color;
use pretty_env_logger::env_logger::fmt::Style;
use pretty_env_logger::env_logger::fmt::StyledValue;
use std::io::Write;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

pub fn set_verbosity_value(value: u8, is_quiet: bool) {
    let env_var_name = format!("{}_LOG", APPLICATION_NAME.to_ascii_uppercase());
    let level = std::env::var(env_var_name).unwrap_or_else(|_| String::from("INFO"));
    let mut level = level.as_str();

    let mut builder = pretty_env_logger::formatted_timed_builder();

    if is_quiet {
        level = "OFF";
    } else {
        match value {
            1 => level = "DEBUG",           // -v
            v if v >= 2 => level = "TRACE", // -vv
            _ => {}
        }
    }

    builder
        .parse_filters(level)
        .format(move |f, record| {
            // This is the same format as the initial one in the pretty_env_logger crate,
            // but only the part with the module name is changed.

            let mut style = f.style();
            let level = colored_level(&mut style, record.level());

            let mut style = f.style();
            let target = if value > 2 {
                let target = format!(" {}", record.target());
                let max_width = max_target_width(&target);
                style.set_bold(true).value(Padded {
                    value: target,
                    width: max_width,
                })
            } else {
                style.value(Padded {
                    value: String::from(""),
                    width: 0,
                })
            };

            let time = f.timestamp_millis();

            writeln!(f, " {} {}{} > {}", time, level, target, record.args(),)
        })
        .try_init()
        .unwrap();
}

// This struct is a copy/paste of the one in pertty_env_logger.
// It's necessary for left-padding the message type.
struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: std::fmt::Display> std::fmt::Display for Padded<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value(" INFO"),
        Level::Warn => style.set_color(Color::Yellow).value(" WARN"),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}
