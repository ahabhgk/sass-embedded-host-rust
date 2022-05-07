/// The wrapper type for all messages sent from the host to the compiler. This
/// provides a `oneof` that makes it possible to determine the type of each
/// inbound message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InboundMessage {
    /// The wrapped message. Mandatory.
    #[prost(oneof="inbound_message::Message", tags="2, 3, 4, 5, 6, 7")]
    pub message: ::core::option::Option<inbound_message::Message>,
}
/// Nested message and enum types in `InboundMessage`.
pub mod inbound_message {
    /// A request for information about the version of the embedded compiler. The
    /// host can use this to provide diagnostic information to the user, to check
    /// which features the compiler supports, or to ensure that it's compatible
    /// with the same protocol version the compiler supports.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct VersionRequest {
        /// This version request's id. Mandatory.
        #[prost(uint32, tag="1")]
        pub id: u32,
    }
    /// A request that compiles an entrypoint to CSS.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CompileRequest {
        /// This compilation's request id. This is included in messages sent from the
        /// compiler to the host. Mandatory.
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// How to format the CSS output.
        #[prost(enumeration="super::OutputStyle", tag="4")]
        pub style: i32,
        /// Whether to generate a source map. Note that this will *not* add a source
        /// map comment to the stylesheet; that's up to the host or its users.
        #[prost(bool, tag="5")]
        pub source_map: bool,
        /// Importers (including load paths on the filesystem) to use when resolving
        /// imports that can't be resolved relative to the file that contains it. Each
        /// importer is checked in order until one recognizes the imported URL.
        #[prost(message, repeated, tag="6")]
        pub importers: ::prost::alloc::vec::Vec<compile_request::Importer>,
        /// Signatures for custom global functions whose behavior is defined by the
        /// host. These must be valid Sass function signatures that could appear in
        /// after `@function` in a Sass stylesheet, such as
        /// `mix($color1, $color2, $weight: 50%)`.
        ///
        /// Compilers must ensure that pure-Sass functions take precedence over
        /// custom global functions. They must also reject any custom function names
        /// that conflict with function names built into the Sass language.
        #[prost(string, repeated, tag="7")]
        pub global_functions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// Whether to use terminal colors in the formatted message of errors and
        /// logs.
        #[prost(bool, tag="8")]
        pub alert_color: bool,
        /// Whether to encode the formatted message of errors and logs in ASCII.
        #[prost(bool, tag="9")]
        pub alert_ascii: bool,
        /// Whether to report all deprecation warnings or only the first few ones.
        /// If this is `false`, the compiler may choose not to send events for
        /// repeated deprecation warnings. If this is `true`, the compiler must emit
        /// an event for every deprecation warning it encounters.
        #[prost(bool, tag="10")]
        pub verbose: bool,
        /// Whether to omit events for deprecation warnings coming from dependencies
        /// (files loaded from a different importer than the input).
        #[prost(bool, tag="11")]
        pub quiet_deps: bool,
        /// Whether to include sources in the generated sourcemap
        #[prost(bool, tag="12")]
        pub source_map_include_sources: bool,
        /// The input stylesheet to parse. Mandatory.
        #[prost(oneof="compile_request::Input", tags="2, 3")]
        pub input: ::core::option::Option<compile_request::Input>,
    }
    /// Nested message and enum types in `CompileRequest`.
    pub mod compile_request {
        /// An input stylesheet provided as plain text, rather than loaded from the
        /// filesystem.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct StringInput {
            /// The contents of the stylesheet.
            #[prost(string, tag="1")]
            pub source: ::prost::alloc::string::String,
            /// The location from which `source` was loaded. If this is empty, it
            /// indicates that the URL is unknown.
            ///
            /// This must be a canonical URL recognized by `importer`, if it's passed.
            #[prost(string, tag="2")]
            pub url: ::prost::alloc::string::String,
            /// The syntax to use to parse `source`.
            #[prost(enumeration="super::super::Syntax", tag="3")]
            pub syntax: i32,
            /// The importer to use to resolve imports relative to `url`.
            #[prost(message, optional, tag="4")]
            pub importer: ::core::option::Option<Importer>,
        }
        /// A wrapper message that represents either a user-defined importer or a
        /// load path on disk. This must be a wrapper because `oneof` types can't be
        /// `repeated`.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Importer {
            /// The possible types of importer. Mandatory.
            #[prost(oneof="importer::Importer", tags="1, 2, 3")]
            pub importer: ::core::option::Option<importer::Importer>,
        }
        /// Nested message and enum types in `Importer`.
        pub mod importer {
            /// The possible types of importer. Mandatory.
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum Importer {
                /// A built-in importer that loads Sass files within the given directory
                /// on disk.
                #[prost(string, tag="1")]
                Path(::prost::alloc::string::String),
                /// A unique ID for a user-defined importer. This ID will be included in
                /// outbound `CanonicalizeRequest` and `ImportRequest` messages to
                /// indicate which importer is being called. The host is responsible for
                /// generating this ID and ensuring that it's unique across all
                /// importers registered for this compilation.
                #[prost(uint32, tag="2")]
                ImporterId(u32),
                /// A unique ID for a special kind of user-defined importer that tells
                /// the compiler where to look for files on the physical filesystem, but
                /// leaves the details of resolving partials and extensions and loading
                /// the file from disk up to the compiler itself.
                ///
                /// This ID will be included in outbound `FileImportRequest` messages to
                /// indicate which importer is being called. The host is responsible for
                /// generating this ID and ensuring that it's unique across all importers
                /// registered for this compilation.
                #[prost(uint32, tag="3")]
                FileImporterId(u32),
            }
        }
        /// The input stylesheet to parse. Mandatory.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Input {
            /// A stylesheet loaded from its contents.
            #[prost(message, tag="2")]
            String(StringInput),
            /// A stylesheet loaded from the given path on the filesystem.
            #[prost(string, tag="3")]
            Path(::prost::alloc::string::String),
        }
    }
    /// A response indicating the result of canonicalizing an imported URL.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CanonicalizeResponse {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The result of canonicalization. Optional. If this is `null`, it indicates
        /// that the importer either did not recognize the URL, or could not find a
        /// stylesheet at the location it referred to.
        #[prost(oneof="canonicalize_response::Result", tags="2, 3")]
        pub result: ::core::option::Option<canonicalize_response::Result>,
    }
    /// Nested message and enum types in `CanonicalizeResponse`.
    pub mod canonicalize_response {
        /// The result of canonicalization. Optional. If this is `null`, it indicates
        /// that the importer either did not recognize the URL, or could not find a
        /// stylesheet at the location it referred to.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Result {
            /// The successfully canonicalized URL. This must be an absolute URL,
            /// including scheme.
            #[prost(string, tag="2")]
            Url(::prost::alloc::string::String),
            /// An error message explaining why canonicalization failed.
            ///
            /// This indicates that a stylesheet was found, but a canonical URL for it
            /// could not be determined. If no stylesheet was found, `result` should be
            /// `null` instead.
            #[prost(string, tag="3")]
            Error(::prost::alloc::string::String),
        }
    }
    /// A response indicating the result of importing a canonical URL.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ImportResponse {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The result of loading the URL. Optional. If this is `null`, it indicates
        /// that the importer either did not recognize the URL, or could not find a
        /// stylesheet at the location it referred to.
        #[prost(oneof="import_response::Result", tags="2, 3")]
        pub result: ::core::option::Option<import_response::Result>,
    }
    /// Nested message and enum types in `ImportResponse`.
    pub mod import_response {
        /// The stylesheet's contents were loaded successfully.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct ImportSuccess {
            /// The text of the stylesheet. Mandatory.
            #[prost(string, tag="1")]
            pub contents: ::prost::alloc::string::String,
            /// The syntax of `contents`. Mandatory.
            #[prost(enumeration="super::super::Syntax", tag="2")]
            pub syntax: i32,
            /// An absolute, browser-accessible URL indicating the resolved location of
            /// the imported stylesheet. Optional.
            ///
            /// This should be a `file:` URL if one is available, but an `http:` URL is
            /// acceptable as well. If no URL is supplied, a `data:` URL is generated
            /// automatically from `contents`.
            ///
            /// If this is provided, it must be an absolute URL, including scheme.
            #[prost(string, tag="3")]
            pub source_map_url: ::prost::alloc::string::String,
        }
        /// The result of loading the URL. Optional. If this is `null`, it indicates
        /// that the importer either did not recognize the URL, or could not find a
        /// stylesheet at the location it referred to.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Result {
            /// The contents of the loaded stylesheet.
            #[prost(message, tag="2")]
            Success(ImportSuccess),
            /// An error message explaining why the URL could not be loaded.
            #[prost(string, tag="3")]
            Error(::prost::alloc::string::String),
        }
    }
    /// A response indicating the result of redirecting a URL to the filesystem.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FileImportResponse {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The result of loading the URL. Optional. A null result indicates that the
        /// importer did not recognize the URL and other importers or load paths
        /// should be tried.
        #[prost(oneof="file_import_response::Result", tags="2, 3")]
        pub result: ::core::option::Option<file_import_response::Result>,
    }
    /// Nested message and enum types in `FileImportResponse`.
    pub mod file_import_response {
        /// The result of loading the URL. Optional. A null result indicates that the
        /// importer did not recognize the URL and other importers or load paths
        /// should be tried.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Result {
            /// The absolute `file:` URL to look for the file on the physical
            /// filesystem.
            ///
            /// The host must ensure that this URL follows the format for an absolute
            /// `file:` URL on the current operating system without a hostname, and the
            /// compiler must verify this to the best of its ability. See
            /// <https://en.wikipedia.org/wiki/File_URI_scheme> for details on the
            /// format.
            ///
            /// The compiler must handle turning this into a canonical URL by resolving
            /// it for partials, file extensions, and index files. The compiler must
            /// then loading the contents of the resulting canonical URL from the
            /// filesystem.
            #[prost(string, tag="2")]
            FileUrl(::prost::alloc::string::String),
            /// An error message explaining why the URL could not be loaded.
            #[prost(string, tag="3")]
            Error(::prost::alloc::string::String),
        }
    }
    /// A response indicating the result of calling a custom Sass function defined
    /// in the host.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FunctionCallResponse {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The IDs of all `Value.ArgumentList`s in `FunctionCallRequest.arguments`
        /// whose keywords were accessed. See `Value.ArgumentList` for details.
        /// Mandatory if `result.success` is set. This may not include the special
        /// value `0` and it may not include multiple instances of the same ID.
        #[prost(uint32, repeated, tag="4")]
        pub accessed_argument_lists: ::prost::alloc::vec::Vec<u32>,
        /// The result of calling the function. Mandatory.
        #[prost(oneof="function_call_response::Result", tags="2, 3")]
        pub result: ::core::option::Option<function_call_response::Result>,
    }
    /// Nested message and enum types in `FunctionCallResponse`.
    pub mod function_call_response {
        /// The result of calling the function. Mandatory.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Result {
            /// The return value of a successful function call.
            #[prost(message, tag="2")]
            Success(super::super::Value),
            /// An error message explaining why the function call failed.
            #[prost(string, tag="3")]
            Error(::prost::alloc::string::String),
        }
    }
    /// The wrapped message. Mandatory.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Message {
        #[prost(message, tag="2")]
        CompileRequest(CompileRequest),
        #[prost(message, tag="3")]
        CanonicalizeResponse(CanonicalizeResponse),
        #[prost(message, tag="4")]
        ImportResponse(ImportResponse),
        #[prost(message, tag="5")]
        FileImportResponse(FileImportResponse),
        #[prost(message, tag="6")]
        FunctionCallResponse(FunctionCallResponse),
        #[prost(message, tag="7")]
        VersionRequest(VersionRequest),
    }
}
/// The wrapper type for all messages sent from the compiler to the host. This
/// provides a `oneof` that makes it possible to determine the type of each
/// outbound message.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutboundMessage {
    /// The wrapped message. Mandatory.
    #[prost(oneof="outbound_message::Message", tags="1, 2, 3, 4, 5, 6, 7, 8")]
    pub message: ::core::option::Option<outbound_message::Message>,
}
/// Nested message and enum types in `OutboundMessage`.
pub mod outbound_message {
    /// A response that contains the version of the embedded compiler.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct VersionResponse {
        /// This version request's id. Mandatory.
        #[prost(uint32, tag="5")]
        pub id: u32,
        /// The version of the embedded protocol, in semver format.
        #[prost(string, tag="1")]
        pub protocol_version: ::prost::alloc::string::String,
        /// The version of the embedded compiler package. This has no guaranteed
        /// format, although compilers are encouraged to use semver.
        #[prost(string, tag="2")]
        pub compiler_version: ::prost::alloc::string::String,
        /// The version of the Sass implementation that the embedded compiler wraps.
        /// This has no guaranteed format, although Sass implementations are
        /// encouraged to use semver.
        #[prost(string, tag="3")]
        pub implementation_version: ::prost::alloc::string::String,
        /// The name of the Sass implementation that the embedded compiler wraps.
        #[prost(string, tag="4")]
        pub implementation_name: ::prost::alloc::string::String,
    }
    /// A response that contains the result of a compilation.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CompileResponse {
        /// The compilation's request id. Mandatory.
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The success or failure result of the compilation. Mandatory.
        #[prost(oneof="compile_response::Result", tags="2, 3")]
        pub result: ::core::option::Option<compile_response::Result>,
    }
    /// Nested message and enum types in `CompileResponse`.
    pub mod compile_response {
        /// A message indicating that the Sass file was successfully compiled to CSS.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CompileSuccess {
            /// The compiled CSS.
            #[prost(string, tag="1")]
            pub css: ::prost::alloc::string::String,
            /// The JSON-encoded source map, or the empty string if
            /// `CompileRequest.source_map` was `false`.
            ///
            /// The compiler must not add a `"file"` key to this source map. It's the
            /// host's (or the host's user's) responsibility to determine how the
            /// generated CSS can be reached from the source map.
            #[prost(string, tag="2")]
            pub source_map: ::prost::alloc::string::String,
            /// The canonical URLs of all source files loaded during the compilation.
            ///
            /// The compiler must ensure that each canonical URL appears only once in
            /// this list. This must include the entrypoint file's URL if either
            /// `CompileRequest.input.path` or `CompileRequest.StringInput.url` was
            /// passed.
            #[prost(string, repeated, tag="3")]
            pub loaded_urls: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        }
        /// A message indicating that the Sass file could not be successfully
        /// compiled to CSS.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CompileFailure {
            /// A message describing the reason for the failure.
            #[prost(string, tag="1")]
            pub message: ::prost::alloc::string::String,
            /// The span associated with the failure. Mandatory.
            #[prost(message, optional, tag="2")]
            pub span: ::core::option::Option<super::super::SourceSpan>,
            /// The stack trace associated with the failure.
            ///
            /// The empty string indicates that no stack trace is available. Otherwise,
            /// the format of this stack trace is not specified and is likely to be
            /// inconsistent between implementations.
            #[prost(string, tag="3")]
            pub stack_trace: ::prost::alloc::string::String,
            /// A formatted, human-readable string that contains the message, span
            /// (if available), and trace (if available). The format of this string is
            /// not specified and is likely to be inconsistent between implementations.
            #[prost(string, tag="4")]
            pub formatted: ::prost::alloc::string::String,
        }
        /// The success or failure result of the compilation. Mandatory.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Result {
            /// The result of a successful compilation.
            #[prost(message, tag="2")]
            Success(CompileSuccess),
            /// The result of a failed compilation.
            #[prost(message, tag="3")]
            Failure(CompileFailure),
        }
    }
    /// An event indicating that a message should be displayed to the user.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct LogEvent {
        /// The request id for the compilation that triggered the message. Mandatory.
        #[prost(uint32, tag="1")]
        pub compilation_id: u32,
        #[prost(enumeration="super::LogEventType", tag="2")]
        pub r#type: i32,
        /// The text of the message.
        #[prost(string, tag="3")]
        pub message: ::prost::alloc::string::String,
        /// The span associated with this message. Optional.
        #[prost(message, optional, tag="4")]
        pub span: ::core::option::Option<super::SourceSpan>,
        /// The stack trace associated with this message.
        ///
        /// The empty string indicates that no stack trace is available. Otherwise,
        /// the format of this stack trace is not specified and is likely to be
        /// inconsistent between implementations.
        #[prost(string, tag="5")]
        pub stack_trace: ::prost::alloc::string::String,
        /// A formatted, human-readable string that contains the message, span (if
        /// available), and trace (if available). The format of this string is not
        /// specified and is likely to be inconsistent between implementations.
        #[prost(string, tag="6")]
        pub formatted: ::prost::alloc::string::String,
    }
    /// A request for a custom importer to convert an imported URL to its canonical
    /// format.
    ///
    /// If the URL is not recognized by this importer, or if no stylesheet is found
    /// at that URL, `CanonicalizeResponse.result` must be `null`. Otherwise, the
    /// importer must return an absolute URL, including a scheme.
    ///
    /// > The host's documentation should encourage the use of file importers (via
    /// > `CompileRequest.Importer.file_importer_id`, `FileImportRequest`, and
    /// > `FileImportResponse`) for any importers that simply refer to files on
    /// > disk. This will allow Sass to handle the logic of resolving partials,
    /// > file extensions, and index files.
    ///
    /// If Sass has already loaded a stylesheet with the returned canonical URL, it
    /// re-uses the existing parse tree. This means that importers must ensure that
    /// the same canonical URL always refers to the same stylesheet, *even across
    /// different importers*. Importers must also ensure that any canonicalized
    /// URLs they return can be passed back to `CanonicalizeRequest` and will be
    /// returned unchanged.
    ///
    /// If this importer's URL format supports file extensions, it should
    /// canonicalize them the same way as the default filesystem importer:
    ///
    /// * The importer should look for stylesheets by adding the prefix `_` to the
    ///   URL's basename, and by adding the extensions `.sass` and `.scss` if the
    ///   URL doesn't already have one of those extensions. For example, if the URL
    ///   was `foo/bar/baz`, the importer would look for:
    ///
    ///   * `foo/bar/baz.sass`
    ///   * `foo/bar/baz.scss`
    ///   * `foo/bar/_baz.sass`
    ///   * `foo/bar/_baz.scss`
    ///
    ///   If the URL was foo/bar/baz.scss, the importer would just look for:
    ///
    ///   * `foo/bar/baz.scss`
    ///   * `foo/bar/_baz.scss`
    ///
    ///   If the importer finds a stylesheet at more than one of these URLs, it
    ///   should respond with a `CanonicalizeResponse.result.error` indicating that
    ///   the import is ambiguous. Note that if the extension is explicitly
    ///   specified, a stylesheet with another extension may exist without error.
    ///
    /// * If none of the possible paths is valid, the importer should perform the
    ///   same resolution on the URL followed by `/index`. In the example above, it
    ///   would look for:
    ///
    ///   * `foo/bar/baz/_index.sass`
    ///   * `foo/bar/baz/index.sass`
    ///   * `foo/bar/baz/_index.scss`
    ///   * `foo/bar/baz/index.scss`
    ///
    ///   As above, if the importer finds a stylesheet at more than one of these
    ///   URLs, it should respond with a `CanonicalizeResponse.result.error`
    ///   indicating that the import is ambiguous.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CanonicalizeRequest {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The request id for the compilation that triggered the message. Mandatory.
        #[prost(uint32, tag="2")]
        pub compilation_id: u32,
        /// The unique ID of the importer being invoked. This must match an importer
        /// ID passed to this compilation in `CompileRequest.importers` or
        /// `CompileRequest.input.string.importer`. Mandatory.
        #[prost(uint32, tag="3")]
        pub importer_id: u32,
        /// The URL of the import to be canonicalized. This may be either absolute or
        /// relative.
        ///
        /// When loading a URL, the compiler must first try resolving that URL
        /// relative to the canonical URL of the current file, and canonicalizing the
        /// result using the importer that loaded the current file. If this returns
        /// `null`, the compiler must then try canonicalizing the original URL with
        /// each importer in order until one returns something other than `null`.
        /// That is the result of the import.
        #[prost(string, tag="4")]
        pub url: ::prost::alloc::string::String,
        //// Whether this request comes from an `@import` rule.
        ////
        //// When evaluating `@import` rules, URLs should canonicalize to an
        //// [import-only file] if one exists for the URL being canonicalized.
        //// Otherwise, canonicalization should be identical for `@import` and `@use`
        //// rules.
        ////
        //// [import-only file]: <https://sass-lang.com/documentation/at-rules/import#import-only-files>
        #[prost(bool, tag="5")]
        pub from_import: bool,
    }
    /// A request for a custom importer to load the contents of a stylesheet.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ImportRequest {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The request id for the compilation that triggered the message. Mandatory.
        #[prost(uint32, tag="2")]
        pub compilation_id: u32,
        /// The unique ID of the importer being invoked. This must match an
        /// `Importer.importer_id` passed to this compilation in
        /// `CompileRequest.importers` or `CompileRequest.input.string.importer`.
        /// Mandatory.
        #[prost(uint32, tag="3")]
        pub importer_id: u32,
        /// The canonical URL of the import. This is guaranteed to be a URL returned
        /// by a `CanonicalizeRequest` to this importer.
        #[prost(string, tag="4")]
        pub url: ::prost::alloc::string::String,
    }
    /// A request for a custom filesystem importer to load the contents of a
    /// stylesheet.
    ///
    /// A filesystem importer is represented in the compiler as an \[importer\]. When
    /// the importer is invoked with a string `string`:
    ///
    /// \[importer\]: <https://github.com/sass/sass/tree/main/spec/modules.md#importer>
    ///
    /// * If `string` is an absolute URL whose scheme is `file`:
    ///
    ///   * Let `url` be string.
    ///
    /// * Otherwise:
    ///
    ///   * Let `fromImport` be `true` if the importer is being run for an
    ///     `@import` and `false` otherwise.
    ///
    ///   * Let `response` be the result of sending a `FileImportRequest` with
    ///     `string` as its `url` and `fromImport` as `from_import`.
    ///
    ///   * If `response.result` is null, return null.
    ///
    ///   * Otherwise, if `response.result.error` is set, throw an error.
    ///
    ///   * Otherwise, let `url` be `response.result.file_url`.
    ///
    /// * Let `resolved` be the result of [resolving `url`].
    ///
    /// * If `resolved` is null, return null.
    ///
    /// * Let `text` be the contents of the file at `resolved`.
    ///
    /// * Let `syntax` be:
    ///   * "scss" if `url` ends in `.scss`.
    ///   * "indented" if `url` ends in `.sass`.
    ///   * "css" if `url` ends in `.css`.
    ///
    ///   > The algorithm for resolving a `file:` URL guarantees that `url` will have
    ///   > one of these extensions.
    ///
    /// * Return `text`, `syntax`, and `resolved`.
    ///
    /// [resolving `url`]: <https://github.com/sass/sass/tree/main/spec/modules.md#resolving-a-file-url>
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FileImportRequest {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The request id for the compilation that triggered the message. Mandatory.
        #[prost(uint32, tag="2")]
        pub compilation_id: u32,
        /// The unique ID of the importer being invoked. This must match an
        /// `Importer.file_importer_id` passed to this compilation in
        /// `CompileRequest.importers` or `CompileRequest.input.string.importer`.
        /// Mandatory.
        #[prost(uint32, tag="3")]
        pub importer_id: u32,
        /// The (non-canonicalized) URL of the import.
        #[prost(string, tag="4")]
        pub url: ::prost::alloc::string::String,
        //// Whether this request comes from an `@import` rule.
        ////
        //// When evaluating `@import` rules, filesystem importers should load an
        //// [import-only file] if one exists for the URL being canonicalized.
        //// Otherwise, canonicalization should be identical for `@import` and `@use`
        //// rules.
        ////
        //// [import-only file]: <https://sass-lang.com/documentation/at-rules/import#import-only-files>
        #[prost(bool, tag="5")]
        pub from_import: bool,
    }
    /// A request to invoke a custom Sass function and return its result.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct FunctionCallRequest {
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The request id for the compilation that triggered the message. Mandatory.
        #[prost(uint32, tag="2")]
        pub compilation_id: u32,
        /// The arguments passed to the function, in the order they appear in the
        /// function signature passed to `CompileRequest.global_functions`. Mandatory.
        ///
        /// The compiler must ensure that a valid number of arguments are passed for
        /// the given signature, that default argument values are instantiated
        /// appropriately, and that variable argument lists (`$args...`) are passed
        /// as `Value.ArgumentList`s.
        #[prost(message, repeated, tag="5")]
        pub arguments: ::prost::alloc::vec::Vec<super::Value>,
        /// An identifier that indicates which function to invoke. Mandatory.
        #[prost(oneof="function_call_request::Identifier", tags="3, 4")]
        pub identifier: ::core::option::Option<function_call_request::Identifier>,
    }
    /// Nested message and enum types in `FunctionCallRequest`.
    pub mod function_call_request {
        /// An identifier that indicates which function to invoke. Mandatory.
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Identifier {
            /// The name of the function to invoke.
            ///
            /// This must match the name of a function signature the host passed to the
            /// corresponding `CompileRequest.global_functions` call, including hyphens
            /// and underscores.
            #[prost(string, tag="3")]
            Name(::prost::alloc::string::String),
            /// The opaque ID of the function to invoke.
            ///
            /// This must match the ID of a `Value.HostFunction` that the host passed
            /// to the compiler.
            #[prost(uint32, tag="4")]
            FunctionId(u32),
        }
    }
    /// The wrapped message. Mandatory.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Message {
        #[prost(message, tag="1")]
        Error(super::ProtocolError),
        #[prost(message, tag="2")]
        CompileResponse(CompileResponse),
        #[prost(message, tag="3")]
        LogEvent(LogEvent),
        #[prost(message, tag="4")]
        CanonicalizeRequest(CanonicalizeRequest),
        #[prost(message, tag="5")]
        ImportRequest(ImportRequest),
        #[prost(message, tag="6")]
        FileImportRequest(FileImportRequest),
        #[prost(message, tag="7")]
        FunctionCallRequest(FunctionCallRequest),
        #[prost(message, tag="8")]
        VersionResponse(VersionResponse),
    }
}
/// An error reported when an endpoint violates the embedded Sass protocol.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProtocolError {
    #[prost(enumeration="ProtocolErrorType", tag="1")]
    pub r#type: i32,
    /// The ID of the request that had an error. This MUST be `4294967295` if the
    /// request ID couldn't be determined, or if the error is being reported for a
    /// response or an event.
    #[prost(uint32, tag="2")]
    pub id: u32,
    /// A human-readable message providing more detail about the error.
    #[prost(string, tag="3")]
    pub message: ::prost::alloc::string::String,
}
/// A chunk of a source file.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SourceSpan {
    /// The text covered by the source span. Compilers must guarantee that this is
    /// the text between `start.offset` and `end.offset` in the source file
    /// referred to by `url`.
    #[prost(string, tag="1")]
    pub text: ::prost::alloc::string::String,
    /// The location of the first character in this span. Mandatory.
    #[prost(message, optional, tag="2")]
    pub start: ::core::option::Option<source_span::SourceLocation>,
    /// The location of the first character after this span. Optional.
    ///
    /// If this is omitted, it indicates that the span is empty and points
    /// immediately before `start`. In that case, `text` must be empty.
    ///
    /// This must not point to a location before `start`.
    #[prost(message, optional, tag="3")]
    pub end: ::core::option::Option<source_span::SourceLocation>,
    /// The URL of the file to which this span refers.
    ///
    /// This may be empty, indicating that the span refers to a
    /// `CompileRequest.StringInput` file that doesn't specify a URL.
    #[prost(string, tag="4")]
    pub url: ::prost::alloc::string::String,
    /// Additional source text surrounding this span.
    ///
    /// If this isn't empty, it must contain `text`. Furthermore, `text` must begin
    /// at column `start.column` of a line in `context`.
    ///
    /// This usually contains the full lines the span begins and ends on if the
    /// span itself doesn't cover the full lines.
    #[prost(string, tag="5")]
    pub context: ::prost::alloc::string::String,
}
/// Nested message and enum types in `SourceSpan`.
pub mod source_span {
    /// A single point in a source file.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SourceLocation {
        /// The 0-based offset of this location within the source file. Mandatory.
        #[prost(uint32, tag="1")]
        pub offset: u32,
        /// The 0-based line number of this location within the source file.
        /// Mandatory.
        #[prost(uint32, tag="2")]
        pub line: u32,
        /// The 0-based column number of this location within its line. Mandatory.
        #[prost(uint32, tag="3")]
        pub column: u32,
    }
}
/// A SassScript value, passed to and returned by functions.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Value {
    /// The value itself. Mandatory.
    ///
    /// This is wrapped in a message type rather than used directly to reduce
    /// repetition, and because oneofs can't be repeated.
    #[prost(oneof="value::Value", tags="1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12")]
    pub value: ::core::option::Option<value::Value>,
}
/// Nested message and enum types in `Value`.
pub mod value {
    /// A SassScript string value.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct String {
        /// The contents of the string. Mandatory.
        #[prost(string, tag="1")]
        pub text: ::prost::alloc::string::String,
        /// Whether the string is quoted or unquoted. Mandatory.
        #[prost(bool, tag="2")]
        pub quoted: bool,
    }
    /// A SassScript number value.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Number {
        /// The number's numeric value. Mandatory.
        #[prost(double, tag="1")]
        pub value: f64,
        /// The number's numerator units.
        ///
        /// The endpoint sending the number must ensure that no numerator units are
        /// \[compatible][\] with any denominator units. Such compatible units must be
        /// simplified away according to the multiplicative factor between them
        /// defined in the CSS Values and Units spec.
        ///
        /// \[compatible\]: <https://www.w3.org/TR/css-values-4/#compat>
        #[prost(string, repeated, tag="2")]
        pub numerators: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// The number's denominator units.
        #[prost(string, repeated, tag="3")]
        pub denominators: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }
    /// A SassScript color value, represented as red, green, and blue channels.
    ///
    /// All Sass color values can be equivalently represented as `RgbColor`,
    /// `HslColor`, and `HwbColor` messages without loss of color information that
    /// can affect CSS rendering. As such, either endpoint may choose to send any
    /// color value as any one of these three messages.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RgbColor {
        /// The color's red channel. Mandatory. May not be above 255.
        #[prost(uint32, tag="1")]
        pub red: u32,
        /// The color's green channel. Mandatory. May not be above 255.
        #[prost(uint32, tag="2")]
        pub green: u32,
        /// The color's blue channel. Mandatory. May not be above 255.
        #[prost(uint32, tag="3")]
        pub blue: u32,
        /// The color's alpha channel. Mandatory. Must be between 0 and 1,
        /// inclusive.
        #[prost(double, tag="4")]
        pub alpha: f64,
    }
    /// A SassScript color value, represented as hue, saturation, and lightness channels.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HslColor {
        /// The color's hue. Mandatory.
        #[prost(double, tag="1")]
        pub hue: f64,
        /// The color's percent saturation. Mandatory. Must be between 0 and 100,
        /// inclusive.
        #[prost(double, tag="2")]
        pub saturation: f64,
        /// The color's percent lightness. Mandatory. Must be between 0 and 100,
        /// inclusive.
        #[prost(double, tag="3")]
        pub lightness: f64,
        /// The color's alpha channel. Mandatory. Must be between 0 and 1,
        /// inclusive.
        #[prost(double, tag="4")]
        pub alpha: f64,
    }
    /// A SassScript color value, represented as hue, whiteness, and blackness
    /// channels.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HwbColor {
        /// The color's hue. Mandatory.
        #[prost(double, tag="1")]
        pub hue: f64,
        /// The color's percent whiteness. Mandatory. Must be between 0 and 100,
        /// inclusive. The sum of `whiteness` and `blackness` must not exceed 100.
        #[prost(double, tag="2")]
        pub whiteness: f64,
        /// The color's percent blackness. Mandatory. Must be between 0 and 100,
        /// inclusive. The sum of `whiteness` and `blackness` must not exceed 100.
        #[prost(double, tag="3")]
        pub blackness: f64,
        /// The color's alpha channel. Mandatory. Must be between 0 and 1,
        /// inclusive.
        #[prost(double, tag="4")]
        pub alpha: f64,
    }
    /// A SassScript list value.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct List {
        /// The type of separator for this list. Mandatory.
        #[prost(enumeration="super::ListSeparator", tag="1")]
        pub separator: i32,
        /// Whether this list has square brackets. Mandatory.
        #[prost(bool, tag="2")]
        pub has_brackets: bool,
        /// The elements of this list.
        #[prost(message, repeated, tag="3")]
        pub contents: ::prost::alloc::vec::Vec<super::Value>,
    }
    /// A SassScript map value.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Map {
        /// The entries in this map. The sending endpoint must guarantee that no two
        /// entries have the same key.
        #[prost(message, repeated, tag="1")]
        pub entries: ::prost::alloc::vec::Vec<map::Entry>,
    }
    /// Nested message and enum types in `Map`.
    pub mod map {
        /// A single key/value pair in the map.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Entry {
            /// The key this entry is associated with. Mandatory.
            #[prost(message, optional, tag="1")]
            pub key: ::core::option::Option<super::super::Value>,
            /// The value associated with this key. Mandatory.
            #[prost(message, optional, tag="2")]
            pub value: ::core::option::Option<super::super::Value>,
        }
    }
    /// A first-class function defined in the compiler. New `CompilerFunction`s may
    /// only be created by the compiler, but the host may pass `CompilerFunction`s
    /// back to the compiler as long as their IDs match IDs of functions received
    /// by the host during that same compilation.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct CompilerFunction {
        /// A unique ID for this function. The compiler is responsible for generating
        /// this ID and ensuring it's unique across all functions passed to the host
        /// for this compilation. Mandatory.
        #[prost(uint32, tag="1")]
        pub id: u32,
    }
    /// An anonymous custom function defined in the host. New `HostFunction`s may
    /// only be created by the host, and `HostFunction`s may *never* be passed from
    /// the compiler to the host. The compiler must instead pass a
    /// `CompilerFunction` that wraps the `HostFunction`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct HostFunction {
        /// A unique ID for this function. The compiler must pass this ID as
        /// `OutboundRequest.FunctionCallRequest.id` when invoking this function. The
        /// host is responsible for generating this ID and ensuring it's unique
        /// across all functions for *all* compilations. Mandatory.
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The signature for this function. Mandatory.
        ///
        /// If this isn't a valid Sass function signature that could appear after
        /// `@function` in a Sass stylesheet (such as `mix($color1, $color2, $weight:
        /// 50%)`), the compiler must treat the function's return value as invalid.
        ///
        /// > This ensures that the host doesn't need to be able to correctly parse
        /// > the entire function declaration syntax.
        ///
        /// The compiler may not invoke the function by its name, since it's not
        /// guaranteed to be globally unique. However, it may use the name to
        /// generate the string representation of this function.
        #[prost(string, tag="2")]
        pub signature: ::prost::alloc::string::String,
    }
    /// A SassScript argument list value. This represents rest arguments passed to
    /// a function's `$arg...` parameter. Unlike a normal `List`, an argument list
    /// has an associated keywords map which tracks keyword arguments passed in
    /// alongside positional arguments.
    ///
    /// For each `ArgumentList` in `FunctionCallRequest.arguments` (including those
    /// nested within `List`s and `Map`s), the host must track whether its keyword
    /// arguments were accessed by the user. If they were, it must add its
    /// `ArgumentList.id` to `FunctionCallResponse.accessed_argument_lists`.
    ///
    /// The compiler must treat every `ArgumentList` whose `ArgumentList.id`
    /// appears in `FunctionCallResponse.accessed_argument_lists` as though it had
    /// been passed to `meta.keywords()`.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ArgumentList {
        /// An ID for this argument list that's unique within the scope of a given
        /// `FunctionCallRequest`.
        ///
        /// The special ID `0` is reserved for `ArgumentList`s created by the host,
        /// and may not be used by the compiler. These `ArgumentList`s do not need to
        /// have their IDs added to `FunctionCallResponse.accessed_argument_lists`,
        /// and the compiler should treat them as though their keywords have always
        /// been accessed.
        #[prost(uint32, tag="1")]
        pub id: u32,
        /// The type of separator for this list. The compiler must set this, but
        /// the host may omit it for `ArgumentList`s that were originally created by
        /// the compiler (that is, those with a non-0 ID).
        #[prost(enumeration="super::ListSeparator", tag="2")]
        pub separator: i32,
        /// The argument list's positional contents. The compiler must set this, but
        /// the host may omit it for `ArgumentList`s that were originally created by
        /// the compiler (that is, those with a non-0 ID).
        #[prost(message, repeated, tag="3")]
        pub contents: ::prost::alloc::vec::Vec<super::Value>,
        /// The argument list's keywords. The compiler must set this, but the host
        /// may omit it for `ArgumentList`s that were originally created by the
        /// compiler (that is, those with a non-0 ID).
        #[prost(map="string, message", tag="4")]
        pub keywords: ::std::collections::HashMap<::prost::alloc::string::String, super::Value>,
    }
    /// A SassScript calculation value. The compiler must send fully \[simplified\]
    /// calculations, meaning that simplifying it again will produce the same
    /// calculation. The host is not required to simplify calculations.
    ///
    /// \[simplified\]: <https://github.com/sass/sass/tree/main/spec/types/calculation.md#simplifying-a-calculation>
    ///
    /// The compiler must simplify any calculations it receives from the host
    /// before returning them from a function. If this simplification produces an
    /// error, it should be treated as though the function call threw that error.
    /// It should *not* be treated as a protocol error.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Calculation {
        /// The calculation's name. Mandatory. The host may only set this to names
        /// that the Sass specification uses to create calculations.
        #[prost(string, tag="1")]
        pub name: ::prost::alloc::string::String,
        /// The calculation's arguments. Mandatory. The host must use exactly the
        /// number of arguments used by the Sass specification for calculations with
        /// the given `name`.
        #[prost(message, repeated, tag="2")]
        pub arguments: ::prost::alloc::vec::Vec<calculation::CalculationValue>,
    }
    /// Nested message and enum types in `Calculation`.
    pub mod calculation {
        /// A single component of a calculation expression.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CalculationValue {
            /// The value of the component. Mandatory.
            #[prost(oneof="calculation_value::Value", tags="1, 2, 3, 4, 5")]
            pub value: ::core::option::Option<calculation_value::Value>,
        }
        /// Nested message and enum types in `CalculationValue`.
        pub mod calculation_value {
            /// The value of the component. Mandatory.
            #[derive(Clone, PartialEq, ::prost::Oneof)]
            pub enum Value {
                #[prost(message, tag="1")]
                Number(super::super::Number),
                /// An unquoted string, as from a function like `var()` or `env()`.
                #[prost(string, tag="2")]
                String(::prost::alloc::string::String),
                /// An unquoted string as created by interpolation for
                /// backwards-compatibility with older Sass syntax.
                #[prost(string, tag="3")]
                Interpolation(::prost::alloc::string::String),
                #[prost(message, tag="4")]
                Operation(::prost::alloc::boxed::Box<super::CalculationOperation>),
                #[prost(message, tag="5")]
                Calculation(super::super::Calculation),
            }
        }
        /// A binary operation that appears in a calculation.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CalculationOperation {
            /// The operator to perform.
            #[prost(enumeration="super::super::CalculationOperator", tag="1")]
            pub operator: i32,
            /// The left-hand side of the operation.
            #[prost(message, optional, boxed, tag="2")]
            pub left: ::core::option::Option<::prost::alloc::boxed::Box<CalculationValue>>,
            /// The right-hand side of the operation.
            #[prost(message, optional, boxed, tag="3")]
            pub right: ::core::option::Option<::prost::alloc::boxed::Box<CalculationValue>>,
        }
    }
    /// The value itself. Mandatory.
    ///
    /// This is wrapped in a message type rather than used directly to reduce
    /// repetition, and because oneofs can't be repeated.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(message, tag="1")]
        String(String),
        #[prost(message, tag="2")]
        Number(Number),
        #[prost(message, tag="3")]
        RgbColor(RgbColor),
        #[prost(message, tag="4")]
        HslColor(HslColor),
        #[prost(message, tag="5")]
        List(List),
        #[prost(message, tag="6")]
        Map(Map),
        #[prost(enumeration="super::SingletonValue", tag="7")]
        Singleton(i32),
        #[prost(message, tag="8")]
        CompilerFunction(CompilerFunction),
        #[prost(message, tag="9")]
        HostFunction(HostFunction),
        #[prost(message, tag="10")]
        ArgumentList(ArgumentList),
        #[prost(message, tag="11")]
        HwbColor(HwbColor),
        #[prost(message, tag="12")]
        Calculation(Calculation),
    }
}
/// Possible ways to format the CSS output. The compiler is not required to
/// support all possible options; if the host requests an unsupported style, the
/// compiler should choose the closest supported style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OutputStyle {
    /// Each selector and declaration is written on its own line.
    Expanded = 0,
    /// The entire stylesheet is written on a single line, with as few characters
    /// as possible.
    Compressed = 1,
}
/// Possible syntaxes for a Sass stylesheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Syntax {
    /// The CSS-superset `.scss` syntax.
    Scss = 0,
    /// The indented `.sass` syntax.
    Indented = 1,
    /// Plain CSS syntax that doesn't support any special Sass features.
    Css = 2,
}
/// The possible types of \[LogEvent\].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum LogEventType {
    /// A warning for something other than a deprecated Sass feature. Often emitted
    /// due to a stylesheet using the `@warn` rule.
    Warning = 0,
    /// A warning indicating that the stylesheet is using a deprecated Sass
    /// feature. Compilers should not add text like "deprecation warning" to
    /// deprecation warnings; it's up to the host to determine how to signal that
    /// to the user.
    DeprecationWarning = 1,
    /// A message generated by the user for their own debugging purposes.
    Debug = 2,
}
/// Potential types of protocol errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ProtocolErrorType {
    /// A message was received that couldn't be decoded as an `InboundMessage` (for
    /// the compiler) or `OutboundMessage` (for the host).
    Parse = 0,
    /// A message was received that violated a documented restriction, such as not
    /// providing a mandatory field.
    Params = 1,
    /// Something unexpected went wrong within the endpoint.
    Internal = 2,
}
/// Different types of separators a list can have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ListSeparator {
    /// List elements are separated by a comma.
    Comma = 0,
    /// List elements are separated by whitespace.
    Space = 1,
    /// List elements are separated by a forward slash.
    Slash = 2,
    /// The list's separator hasn't yet been determined. This is only allowed for
    /// singleton and empty lists.
    ///
    /// Singleton lists and empty lists don't have separators defined. This means
    /// that list functions will prefer other lists' separators if possible.
    Undecided = 3,
}
/// Singleton SassScript values that have no internal state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SingletonValue {
    /// The SassScript boolean true value.
    True = 0,
    /// The SassScript boolean false value.
    False = 1,
    /// The SassScript null value.
    Null = 2,
}
/// An operator used in a calculation value's operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CalculationOperator {
    /// The addition operator.
    Plus = 0,
    /// The subtraction operator.
    Minus = 1,
    /// The multiplication operator.
    Times = 2,
    /// The division operator.
    Divide = 3,
}
