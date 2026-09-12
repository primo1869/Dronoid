#![forbid(unsafe_code)]

use colored::Color;
use colored::Colorize;
use std::time::Instant;
use tracing::{
    Event, Level,
    field::{Field, Visit},
};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    EnvFilter, Layer, Registry,
    layer::{Context, SubscriberExt},
};

#[derive(Default)]
struct MessageVisitor {
    message: String,
    sender: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "sender" {
            self.sender = value.to_string();
        }
    }
}

struct CustomLayer {
    time_mark: Instant,
}

impl CustomLayer {
    fn new() -> Self {
        Self {
            time_mark: Instant::now(),
        }
    }
}

impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let color = match *event.metadata().level() {
            Level::TRACE => Color::BrightBlack,
            Level::DEBUG => Color::Green,
            Level::INFO => Color::White,
            Level::WARN => Color::Yellow,
            Level::ERROR => Color::Red,
        };

        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let formatted = format!(
            "[{:>7.3}][{:^15}]{}",
            (Instant::now() - self.time_mark).as_secs_f32(),
            event.metadata().target(),
            visitor.message
        );
        println!("{}", formatted.color(color));
    }
}

pub fn init_logger() {
    let _ = Registry::default()
        .with(CustomLayer::new())
        .with(EnvFilter::from_default_env())
        .try_init();
}
