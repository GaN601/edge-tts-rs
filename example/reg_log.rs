use chrono::{Local, Utc};
use std::error::Error;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;
use tracing_subscriber::FmtSubscriber;

struct CustomTimeFormatter;

impl FormatTime for CustomTimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let local_time = Local::now();
        write!(w, "{}", local_time.format("%Y-%m-%d %H:%M:%S"))
    }
}

pub fn reg_log() -> Result<(), Box<dyn Error>> {
    LogTracer::init()?;
    let layer = FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_timer(CustomTimeFormatter {})
        .finish();
    tracing::subscriber::set_global_default(layer)?;
    Ok(())
}
