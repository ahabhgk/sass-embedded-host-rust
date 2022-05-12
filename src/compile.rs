use crate::{
  api::{CompileResult, Exception, Options, Result, StringOptions},
  compiler::Embedded,
  compiler_path,
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  pb::{inbound_message::CompileRequest, outbound_message::compile_response},
  Error,
};

pub async fn compile(
  path: String,
  mut options: Options,
) -> Result<CompileResult> {
  let mut importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_path(path, &mut importers, &options);
  let embedded = Embedded::new(compiler_path::compiler_path().unwrap());
  let response = embedded.compile(request, &importers, &logger).await?;

  let res = response.result.ok_or_else(|| {
    Error::Compile(
      "OutboundMessage.CompileResponse.result is not set".to_string(),
    )
  })?;
  match res {
    compile_response::Result::Success(success) => Ok(success.into()),
    compile_response::Result::Failure(failure) => {
      Err(Exception::new(failure).into())
    }
  }
}

pub async fn compile_string(
  source: String,
  mut options: Options,
  string_options: StringOptions,
) -> Result<CompileResult> {
  let mut importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_string(
    source,
    &mut importers,
    &options,
    string_options,
  );
  let embedded = Embedded::new(compiler_path::compiler_path().unwrap());
  let response = embedded.compile(request, &importers, &logger).await?;

  let res = response.result.ok_or_else(|| {
    Error::Compile(
      "OutboundMessage.CompileResponse.result is not set".to_string(),
    )
  })?;
  match res {
    compile_response::Result::Success(success) => Ok(success.into()),
    compile_response::Result::Failure(failure) => {
      Err(Exception::new(failure).into())
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::api::WithoutImporter;

  use super::*;

  #[tokio::test]
  async fn test_compile_string() {
    let res = compile_string(
      ".foo {a: b}".to_string(),
      Options::default(),
      StringOptions::WithoutImporter(WithoutImporter::default()),
    )
    .await
    .unwrap();
    dbg!(res);
  }
}
