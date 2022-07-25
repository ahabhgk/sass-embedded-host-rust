// use crate::{
//   api::{Options, StringOptions},
//   importer_registry::ImporterRegistry,
// };

pub use sass_embedded_protocol::*;

use self::inbound_message::{compile_request, CompileRequest};

mod sass_embedded_protocol;

// impl CompileRequest {
//   fn new(importers: &ImporterRegistry, options: &Options) -> Self {
//     let mut request = CompileRequest {
//       importers: importers.importers(),
//       global_functions: Vec::new(), // TODO
//       source_map: options.source_map,
//       source_map_include_sources: options.source_map_include_sources,
//       alert_color: options.alert_color.unwrap_or_else(|| {
//         supports_color::on(supports_color::Stream::Stdout).is_some()
//       }),
//       alert_ascii: options.alert_ascii,
//       quiet_deps: options.quiet_deps,
//       ..Default::default()
//     };
//     request.set_style(options.style);
//     request
//   }

//   pub fn with_path(
//     path: String,
//     importers: &ImporterRegistry,
//     options: &Options,
//   ) -> Self {
//     let mut request = Self::new(importers, options);
//     request.input = Some(compile_request::Input::Path(path));
//     request
//   }

//   pub fn with_string(
//     source: String,
//     importers: &mut ImporterRegistry,
//     options: &Options,
//     string_options: StringOptions,
//   ) -> Self {
//     let mut input = compile_request::StringInput {
//       source,
//       ..Default::default()
//     };
//     match string_options {
//       StringOptions::WithImporter(o) => {
//         input.set_syntax(o.syntax);
//         input.url = o.url.to_string();
//         input.importer = Some(importers.register(o.importer));
//       }
//       StringOptions::WithoutImporter(o) => {
//         input.set_syntax(o.syntax);
//         if let Some(url) = o.url {
//           input.url = url.to_string();
//         }
//       }
//     };
//     let mut request = CompileRequest::new(importers, options);
//     request.input = Some(compile_request::Input::String(input));
//     request
//   }
// }

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
