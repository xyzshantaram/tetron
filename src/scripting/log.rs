use super::{
    NativeModule,
    utils::{FnOpts, register_fn},
};
use crate::TetronError;
use rhai::{Engine, Module, Scope};
use std::rc::Rc;

const LOG_SCRIPT: &str = r#"
fn trace(s) {
    __tetron_logger_internal("debug", s);
}
fn info(s) {
    __tetron_logger_internal("info", s);
}
fn warn(s) {
    __tetron_logger_internal("warn", s);
}
fn error(s) {
    __tetron_logger_internal("error", s);
}
"#;

fn logger(scope: &str, value: &str) {
    if scope == "info" {
        println!("tetron::log {} {value}", scope.to_ascii_uppercase());
    } else {
        eprintln!("tetron::log {} {value}", scope.to_ascii_uppercase());
    }
}

pub fn log_module(engine: &mut Engine) -> Result<NativeModule, TetronError> {
    let ast = engine
        .compile(LOG_SCRIPT)
        .map_err(|e| TetronError::Other(format!("This should never happen: {e}")))?;

    let mut module = Module::eval_ast_as_new(Scope::new(), &ast, engine)
        .map_err(|e| TetronError::Other(format!("This should never happen: {e}")))?;

    register_fn(
        &mut module,
        "__tetron_logger_internal",
        logger,
        FnOpts::new().global(),
    );

    Ok(("log", Rc::new(module)))
}
