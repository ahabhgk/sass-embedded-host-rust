use crate::{
  protocol::{outbound_message::LogEvent, LogEventType},
  LoggerDebugOptions, LoggerWarnOptions, SassLogger,
};

#[derive(Debug, Default)]
pub struct LoggerRegistry {
  logger: Option<SassLogger>,
}

impl LoggerRegistry {
  pub fn register(&mut self, logger: SassLogger) {
    self.logger = Some(logger);
  }

  pub fn log(&self, event: LogEvent) {
    if let Some(logger) = &self.logger {
      if event.r#type() == LogEventType::Debug {
        logger.debug(
          &event.message,
          &LoggerDebugOptions {
            span: event.span,
            formatted: event.formatted,
          },
        );
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
            formatted: event.formatted,
          },
        );
      }
    } else {
      eprintln!("{}", event.formatted);
    }
  }
}
