use crate::{
  api::{Options, StringOptions},
  importer_registry::ImporterRegistry,
};

pub use sass_embedded_protocol::*;

use self::inbound_message::{
  canonicalize_response, compile_request, CompileRequest,
};

mod sass_embedded_protocol;

impl CompileRequest {
  fn new(importers: &ImporterRegistry, options: Options) -> Self {
    let mut request = CompileRequest::default();
    request.importers = importers.importers();
    request.global_functions = Vec::new(); // TODO
    request.source_map = options.source_map;
    request.source_map_include_sources = options.source_map_include_sources;
    request.alert_color = options.alert_color.unwrap_or_else(|| {
      supports_color::on(supports_color::Stream::Stdout).is_some()
    });
    request.alert_ascii = options.alert_ascii;
    request.quiet_deps = options.quiet_deps;
    request.set_style(options.style);
    request
  }

  pub fn with_string(
    source: String,
    importers: &mut ImporterRegistry,
    options: StringOptions,
  ) -> Self {
    let mut input = compile_request::StringInput::default();
    input.source = source;
    let base = match options {
      StringOptions::WithImporter(o) => {
        input.url = o.url.to_string();
        input.importer = Some(importers.register(o.importer));
        o.base.base
      }
      StringOptions::WithoutImporter(o) => {
        input.set_syntax(o.syntax);
        if let Some(url) = o.url {
          input.url = url.to_string();
        }
        o.base
      }
    };
    let mut request = CompileRequest::new(importers, base);
    request.input = Some(compile_request::Input::String(input));
    request
  }
}

impl compile_request::Importer {
  pub fn new(i: compile_request::importer::Importer) -> Self {
    Self { importer: Some(i) }
  }
}

impl InboundMessage {
  pub fn new(m: inbound_message::Message) -> Self {
    Self { message: Some(m) }
  }
}
