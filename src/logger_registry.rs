use crate::{
  protocol::{outbound_message::LogEvent, LogEventType},
  LoggerDebugOptions, LoggerWarnOptions, SassLogger,
};

#[derive(Debug)]
pub struct LoggerRegistry {
  logger: Option<SassLogger>,
}

impl LoggerRegistry {
  pub fn new(logger: Option<SassLogger>) -> Self {
    Self { logger }
  }

  pub fn log(&self, event: LogEvent) {
    if let Some(logger) = &self.logger {
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
