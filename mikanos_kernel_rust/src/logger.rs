use crate::printk;
use core::fmt;
use core::fmt::Formatter;

static LOG_LEVEL: spin::RwLock<Level> = spin::RwLock::new(Level::Info);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Level::Error => "ERROR",
            Level::Warn => "WARN",
            Level::Info => "INFO",
            Level::Debug => "DEBUG",
            Level::Trace => "TRACE",
        };
        write!(f, "{}", s)
    }
}

pub fn set_level(level: Level) {
    *LOG_LEVEL.write() = level;
}

#[doc(hidden)]
pub fn _log(level: Level, arg: fmt::Arguments) {
    if level <= *LOG_LEVEL.read() {
        printk!("[{}] {}", level, arg);
    }
}

#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::logger::_log($level, format_args!($($arg)*));
    }
}
