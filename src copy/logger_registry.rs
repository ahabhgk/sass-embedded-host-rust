use crate::{
  api::{LoggerDebugOptions, LoggerWarnOptions, SassLogger},
  pb::{outbound_message::LogEvent, LogEventType},
};

pub struct LoggerRegistry {
  loggers: Option<SassLogger>,
}

impl LoggerRegistry {
  pub fn new(loggers: Option<SassLogger>) -> Self {
    Self { loggers }
  }

  pub fn log(&self, event: LogEvent) {
    if let Some(logger) = &self.loggers {
      if event.r#type() == LogEventType::Debug {
        logger.debug(&event.message, &LoggerDebugOptions { span: event.span });
      } else {
        let deprecation = event.r#type() == LogEventType::DeprecationWarning;
        logger.warn(
          &event.message,
          &LoggerWarnOptions {
            span: event.span,
            deprecation,
            stack: if event.stack_trace.is_empty() {
              None
            } else {
              Some(event.stack_trace)
            },
          },
        );
      }
    } else {
      eprintln!("{}", event.formatted);
    }
  }
}
