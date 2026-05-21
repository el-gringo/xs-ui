use log::{Level, Log};

struct Logger;
static LOGGER: Logger = Logger;

impl Log for Logger {
    #[inline]
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        log(record);
    }

    fn flush(&self) {}
}

#[cfg(target_arch = "wasm32")]
fn log(record: &log::Record) {
    use xs_js_sys;

    // pick the console.log() variant for the appropriate logging level
    let console_log = match record.level() {
        Level::Debug => xs_js_sys::log_debug,
        Level::Error => xs_js_sys::log_error,
        Level::Info => xs_js_sys::log_info,
        Level::Trace => xs_js_sys::log_trace,
        Level::Warn => xs_js_sys::log_warn,
    };

    let message = {
        let message = format!(
            "%c{level}%c {file}:{line} %c\n{text}",
            level = record.level(),
            file = record.file().unwrap_or_else(|| record.target()),
            line = record
                .line()
                .map_or_else(|| "[Unknown]".to_string(), |line| line.to_string()),
            text = record.args(),
        );
        message
    };

    #[cfg(feature = "color")]
    {
        let level_style = {
            let color = match record.level() {
                Level::Debug => "blue",
                Level::Error => "darkred",
                Level::Info => "green",
                Level::Trace => "gray",
                Level::Warn => "orange",
            };

            "color: white; padding: 0 3px; background: ".to_owned() + color + ";"
        };
        console_log(&message, &level_style);
    }
    #[cfg(not(feature = "color"))]
    console_log(&message, "");
}

#[cfg(not(target_arch = "wasm32"))]
fn log(record: &log::Record) {
    #[cfg(feature = "color")]
    {
        let color = match record.level() {
            Level::Debug => "\u{1b}[0;34m",
            Level::Info => "\u{1b}[0;32m",
            Level::Error => "\u{1b}[0;31m",
            Level::Trace => "\u{1b}[0;36m",
            Level::Warn => "\u{1b}[0;33m",
        };
        let level = record.level().as_str();
        println!("{color}[{}]\u{1b}[1;0m {}", level, record.args());
    }
    #[cfg(not(feature = "color"))]
    {
        let level = record.level().as_str();
        println!("[{}] {}", level, record.args());
    }
}

/// Initializes the global logger setting `max_log_level` to the given value.
///
/// ## Example
///
/// ```
/// use xs_log::Level;
/// fn main() {
///     xs_log::init_with_level(Level::Debug).expect("error initializing logger");
/// }
/// ```
#[inline]
pub fn init_with_level(level: Level) {
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(level.to_level_filter());
}

/// Initializes the global logger with `max_log_level` set to `Level::Info` (a sensible default).
///
/// ## Example
///
/// ```
/// use xs_log;
/// fn main() {
///     xs_log::init().expect("error initializing logger");
/// }
/// ```
#[inline]
pub fn init() {
    init_with_level(Level::Info);
}
