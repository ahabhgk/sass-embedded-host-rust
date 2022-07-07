use crate::{
  api::{CompileResult, Exception, Options, Result, StringOptions},
  compiler::Embedded,
  compiler_path,
  importer_registry::ImporterRegistry,
  logger_registry::LoggerRegistry,
  pb::{
    inbound_message::CompileRequest,
    outbound_message::{compile_response, CompileResponse},
  },
  Error,
};

pub fn compile_sync(
  path: String,
  mut options: Options,
) -> Result<CompileResult> {
  let exe = exe_path(&options);
  let importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_path(path, &importers, &options);
  let rt = tokio::runtime::Runtime::new().unwrap();
  let response = rt.block_on(async {
    let embedded = Embedded::new(exe);
    let res = embedded.compile(request, &importers, &logger).await?;
    Ok::<CompileResponse, Error>(res)
  })?;

  handle_response(response)
}

pub fn compile_string_sync(
  source: String,
  mut options: Options,
  string_options: StringOptions,
) -> Result<CompileResult> {
  let exe = exe_path(&options);
  let mut importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_string(
    source,
    &mut importers,
    &options,
    string_options,
  );
  let rt = tokio::runtime::Runtime::new().unwrap();
  let response = rt.block_on(async {
    let embedded = Embedded::new(exe);
    let res = embedded.compile(request, &importers, &logger).await?;
    Ok::<CompileResponse, Error>(res)
  })?;

  handle_response(response)
}

pub async fn compile(
  path: String,
  mut options: Options,
) -> Result<CompileResult> {
  let exe = exe_path(&options);
  let importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_path(path, &importers, &options);
  let embedded = Embedded::new(exe);
  let response = embedded.compile(request, &importers, &logger).await?;

  handle_response(response)
}

pub async fn compile_string(
  source: String,
  mut options: Options,
  string_options: StringOptions,
) -> Result<CompileResult> {
  let exe = exe_path(&options);
  let mut importers =
    ImporterRegistry::new(options.importers.take(), options.load_paths.take());
  let logger = LoggerRegistry::new(options.logger.take());

  let request = CompileRequest::with_string(
    source,
    &mut importers,
    &options,
    string_options,
  );
  let embedded = Embedded::new(exe);
  let response = embedded.compile(request, &importers, &logger).await?;

  handle_response(response)
}

fn exe_path(options: &Options) -> String {
  options
    .exe_path
    .clone()
    .unwrap_or_else(|| compiler_path::compiler_path().unwrap())
}

fn handle_response(response: CompileResponse) -> Result<CompileResult> {
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
