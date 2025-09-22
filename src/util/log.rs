/// Sets the function name used in log macros to the given value
#[macro_export]
macro_rules! log_fn_name {
    ($arg:literal) => {
        pub const LOG_FN_NAME: &str = $arg;
    };
}

/// Prints out a blue info message on `stderr`, with a function name prefix.
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_BLUE}info:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

/// Prints out a yellow warning message on `stderr`, with a function name prefix.
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_YELLOW}warn:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

/// Prints out a red error message on `stderr`, with a function name prefix.
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_RED}error:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

/// Prints out a green success message on `stderr`, with a function name prefix.
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        eprintln!("[{}] {ANSI_COLOR_BOLD_GREEN}success:{ANSI_COLOR_RESET} {}", LOG_FN_NAME, format!($($arg)*));
    }};
}

/// Prints out a blue info message on `stderr`, without using a function name prefix.
///
/// "npr" stands for "no prefix".
#[macro_export]
macro_rules! info_npr {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_BLUE, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_BLUE}info:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

/// Prints out a yellow warning message on `stderr`, without using a function name prefix.
///
/// "npr" stands for "no prefix".
#[macro_export]
macro_rules! warn_npr {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_YELLOW, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_YELLOW}warn:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

/// Prints out a red error message on `stderr`, without using a function name prefix.
///
/// "npr" stands for "no prefix".
#[macro_export]
macro_rules! error_npr {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_RED, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_RED}error:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}

/// Prints out a green success message on `stderr`, without using a function name prefix.
///
/// "npr" stands for "no prefix".
#[macro_export]
macro_rules! success_npr {
    ($($arg:tt)*) => {{
        use $crate::util::terminal_colors::{ANSI_COLOR_BOLD_GREEN, ANSI_COLOR_RESET};
        eprintln!("{ANSI_COLOR_BOLD_GREEN}success:{ANSI_COLOR_RESET} {}", format!($($arg)*));
    }};
}
