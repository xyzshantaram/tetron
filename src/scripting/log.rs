use rune::{
    ContextError, Module, compile,
    macros::{MacroContext, TokenStream, quote},
    parse::Parser,
};
use std::sync::atomic::{AtomicU8, Ordering};

use crate::system_log;

/// Global log level that can be changed at runtime
static CURRENT_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Info as u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum LogLevel {
    Off = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
}

impl LogLevel {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "off" => Some(LogLevel::Off),
            "error" => Some(LogLevel::Error),
            "warn" | "warning" => Some(LogLevel::Warn),
            "info" => Some(LogLevel::Info),
            "debug" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Off => "OFF",
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
        }
    }

    fn color(&self) -> &'static str {
        match self {
            LogLevel::Off => "",
            LogLevel::Error => "\x1b[31m", // Red
            LogLevel::Warn => "\x1b[33m",  // Yellow
            LogLevel::Info => "\x1b[32m",  // Green
            LogLevel::Debug => "\x1b[36m", // Cyan
        }
    }
}

// Native logging function that respects the current log level
#[allow(unused)]
#[rune::function(keep)]
fn native_log(level_str: &str, file: &str, line: i64, message: &str) {
    let Some(level) = LogLevel::from_str(level_str) else {
        eprintln!("Invalid log level: {}", level_str);
        return;
    };

    let current_level = LogLevel::from_str(&current_log_level()).unwrap_or(LogLevel::Info);

    // Only log if the message level is <= current log level
    if level <= current_level && current_level != LogLevel::Off {
        let reset = "\x1b[0m"; // Reset color
        let color = level.color();

        println!(
            "tetron::log {color}[{}]{reset} {file}:{line}: {message}",
            level.as_str(),
        );
    }
}

// Function to set the log level at runtime
#[rune::function(keep)]
pub fn level(level: &str) -> bool {
    if let Some(log_level) = LogLevel::from_str(level) {
        CURRENT_LOG_LEVEL.store(log_level as u8, Ordering::Relaxed);
        system_log!("Log level set to: {}", log_level.as_str());
        true
    } else {
        eprintln!(
            "tetron::log Invalid log level '{}'. Valid levels: off, error, warn, info, debug",
            level
        );
        false
    }
}

/// Get the current log level.
fn current_log_level() -> String {
    let level_num = CURRENT_LOG_LEVEL.load(Ordering::Relaxed);
    let level = match level_num {
        0 => LogLevel::Off,
        1 => LogLevel::Error,
        2 => LogLevel::Warn,
        3 => LogLevel::Info,
        4 => LogLevel::Debug,
        _ => LogLevel::Info, // fallback
    };
    level.as_str().to_lowercase()
}

// Macro helper function to create logging macros
fn log_macro(
    level: &'static str,
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    let mut parser = Parser::from_token_stream(stream, cx.input_span());
    let message = parser.parse_all::<rune::ast::Expr>()?;

    let level = cx.lit(level)?;
    let expanded = quote! {
        ::tetron::log::native_log(#level, file!(), line!(), #message)
    };

    Ok(expanded.into_token_stream(cx)?)
}

// Individual logging macros
#[rune::macro_]
pub fn println(
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    log_macro("info", cx, stream)
}

#[rune::macro_]
pub fn info(
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    log_macro("info", cx, stream)
}

#[rune::macro_]
pub fn debug(
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    log_macro("debug", cx, stream)
}

#[rune::macro_]
pub fn error(
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    log_macro("error", cx, stream)
}

#[rune::macro_]
pub fn warn(
    cx: &mut MacroContext<'_, '_, '_>,
    stream: &TokenStream,
) -> compile::Result<TokenStream> {
    log_macro("warn", cx, stream)
}

// Create the tetron::log module
pub fn module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate_item("tetron", ["log"])?;

    module.function_meta(native_log__meta)?;
    module.function_meta(level__meta)?;

    // Register logging macros
    module.macro_meta(println)?;
    module.macro_meta(info)?;
    module.macro_meta(debug)?;
    module.macro_meta(error)?;
    module.macro_meta(warn)?;

    Ok(module)
}
