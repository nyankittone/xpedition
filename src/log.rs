// fucking macros... just copy-paste shit that already works

#[macro_export]
macro_rules! info {
    () => {{
        eprintln!("\x1b[1minfo:\x1b[m");
    }};
    ($fmt:literal) => {{
        eprintln!("\x1b[1minfo:\x1b[m {}", format_args!($fmt));
    }};
    ($fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1minfo:\x1b[m {}", format_args!($fmt, $($arg)*));
    }};
    ($error:expr) => {{
        // typecheck
        let error: std::error::Error = $error;
        eprintln!("\x1b[1minfo:\x1b[m {}", error);
    }};
    ($error:expr, $fmt:literal) => {{
        eprintln!("\x1b[1minfo:\x1b[m {}: {}", format_args!($fmt), $error);
    }};
    ($error:expr, $fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1minfo:\x1b[m {}: {}", format_args!($fmt, $($arg)*), $error);
    }};
}

#[macro_export]
macro_rules! warn {
    () => {{
        eprintln!("\x1b[1;93mwarn:\x1b[m");
    }};
    ($fmt:literal) => {{
        eprintln!("\x1b[1;93mwarn:\x1b[m {}", format_args!($fmt));
    }};
    ($fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1;93mwarn:\x1b[m {}", format_args!($fmt, $($arg)*));
    }};
    ($error:expr) => {{
        let error = $error;
        eprintln!("\x1b[1;93mwarn:\x1b[m {}", error);
    }};
    ($error:expr, $fmt:literal) => {{
        eprintln!("\x1b[1;93mwarn:\x1b[m {}: {}", format_args!($fmt), $error);
    }};
    ($error:expr, $fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1;93mwarn:\x1b[m {}: {}", format_args!($fmt, $($arg)*), $error);
    }};
}

#[macro_export]
macro_rules! fatal {
    () => {{
        eprintln!("\x1b[1;91mfatal:\x1b[m");
    }};
    ($fmt:literal) => {{
        eprintln!("\x1b[1;91mfatal:\x1b[m {}", format_args!($fmt));
    }};
    ($fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1;91mfatal:\x1b[m {}", format_args!($fmt, $($arg)*));
    }};
    ($error:expr) => {{
        // typecheck
        // let error: std::error::Error = $error;
        eprintln!("\x1b[1;91mfatal:\x1b[m {}", $error);
    }};
    ($error:expr, $fmt:literal) => {{
        eprintln!("\x1b[1;91mfatal:\x1b[m {}: {}", format_args!($fmt), $error);
    }};
    ($error:expr, $fmt:literal, $($arg:tt)*) => {{
        eprintln!("\x1b[1;91mfatal:\x1b[m {}: {}", format_args!($fmt, $($arg)*), $error);
    }};
}

