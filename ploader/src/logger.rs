use log::LevelFilter;
use std::env::var;
use thorn::prelude::*;


fn get_level(default: LevelFilter) -> LevelFilter
{
    if let Ok(level) = var("THORN_LOG")
    {
        return match level.to_lowercase().trim()
        {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warning" | "warn" => LevelFilter::Warn,
            "error" | "err" => LevelFilter::Error,
            "none" | "off" | "false" => LevelFilter::Off,
            _ => default,
        };
    }

    default
}


fn get_log_file(default: &str) -> String
{
    var("THORN_LOG_FILE").unwrap_or(default.to_string())
}


pub fn init() -> ThResult<()>
{
    fern::Dispatch::new()
        .format(|out, msg, record| {
            out.finish(format_args!(
                "[{}::{}:{}] {}",
                record.level(),
                record.file().unwrap_or("<unknown>"),
                record.line().map(|l| l.to_string()).unwrap_or("?".into()),
                msg,
            ))
        })
        .level(get_level(LevelFilter::Debug))
        .chain(std::io::stdout())
        .chain(
            fern::log_file(get_log_file(&format!("{}.log", env!("CARGO_CRATE_NAME"))))
                .map_err(|e| ThError::Error(e.to_string()))?,
        )
        .apply()
        .map_err(|e| ThError::Error(e.to_string()))?;

    Ok(())
}
