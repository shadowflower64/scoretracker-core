pub const ANSI_COLOR_RESET: &str = "\x1b[0m";
pub const ANSI_COLOR_BOLD_RED: &str = "\x1b[1;31m";
pub const ANSI_COLOR_BOLD_GREEN: &str = "\x1b[1;32m";
pub const ANSI_COLOR_BOLD_YELLOW: &str = "\x1b[1;33m";
pub const ANSI_COLOR_BOLD_BLUE: &str = "\x1b[1;34m";

#[macro_export]
macro_rules! log_fn_name {
    ($arg:tt) => {
        pub const LOG_FN_NAME: &str = $arg;
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_BLUE}info:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_YELLOW}warn:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_RED}error:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_GREEN}success:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! info_npr {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_BLUE}info:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn_npr {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_YELLOW}warn:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! error_npr {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_RED}error:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! success_npr {
    ($($arg:tt)*) => {{
        use $crate::util::log::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_GREEN}success:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}
