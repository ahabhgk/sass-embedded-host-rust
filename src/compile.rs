use crate::{
  api::{CompileResult, Exception, Result, StringOptions},
  compiler::Embedded,
  compiler_path,
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  pb::{inbound_message::CompileRequest, outbound_message::compile_response},
};

pub async fn compile_string(
  source: String,
  mut options: StringOptions,
) -> Result<CompileResult> {
  let base = options.get_options_mut();
  let mut importers =
    ImporterRegistry::new(base.importers.take(), base.load_paths.take());
  let logger = LoggerRegistry::new(base.logger.take());

  let request = CompileRequest::with_string(source, &mut importers, options);
  let mut embedded = Embedded::new(compiler_path::compiler_path().unwrap());
  let response = embedded
    .send_compile_request(request, importers, logger)
    .await?;
  match response.result.unwrap() {
    compile_response::Result::Success(success) => {
      let css = success.css;
      let source_map = success.source_map;
      let loaded_urls = success.loaded_urls;
      Ok(CompileResult {
        css,
        source_map: Some(source_map),
        loaded_urls,
      })
    }
    compile_response::Result::Failure(failure) => {
      Err(Exception::new(failure).into())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::api::StringOptionsWithoutImporter;

  use super::*;

  #[tokio::test]
  async fn test_compile_string() {
    let res = compile_string(
      ".foo {a: b}".to_string(),
      StringOptions::WithoutImporter(StringOptionsWithoutImporter::default()),
    )
    .await
    .unwrap();
    dbg!(res);
  }
}
