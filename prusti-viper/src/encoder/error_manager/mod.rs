use encoder::vir::Position;
use std::collections::HashMap;
use syntax::codemap::Span;
use syntax_pos::MultiSpan;
use uuid::Uuid;
use viper::VerificationError;
use syntax::codemap::CodeMap;

/// The cause of a panic!()
#[derive(Clone,Debug)]
pub enum PanicCause {
    /// Unknown cause
    Unknown,
    /// Caused by a panic!()
    Panic,
    /// Caused by an assert!()
    Assert,
    /// Caused by an unreachable!()
    Unreachable,
    /// Caused by an unimplemented!()
    Unimplemented,
}

/// In case of verification error, this enum will contain all the information (span, ...)
/// required to report the error in the compiler.
#[derive(Clone,Debug)]
pub enum ErrorCtxt {
    /// A Viper `assert false` that encodes a Rust panic
    Panic(PanicCause),
    /// A Viper `exhale expr` that encodes the call of a Rust procedure with precondition `expr`
    ExhalePrecondition,
    /// A Viper `exhale expr` that encodes the end of a Rust procedure with postcondition `expr`
    ExhalePostcondition,
    /// A Viper `exhale expr` that exhales the permissions of a loop invariant `expr`
    ExhaleLoopInvariant,
    /// A Viper `assert expr` that asserts the functional specification of a loop invariant `expr`
    AssertLoopInvariant,
    /// A Viper `assert false` that encodes the failure (panic) of an `assert` Rust terminator
    /// Arguments: the message of the Rust assertion
    AssertTerminator(String),
    /// A Viper `assert false` that encodes an `abort` Rust terminator
    AbortTerminator,
    /// A Viper `assert false` that encodes an `unreachable` Rust terminator
    UnreachableTerminator,
    /// An error that should never happen
    Unexpected,
}

/// The Rust error that will be reported from the compiler
#[derive(Clone,Debug)]
pub struct CompilerError {
    pub id: String,
    pub message: String,
    pub span: MultiSpan,
}

impl CompilerError {
    pub fn new<S1: ToString, S2: ToString>(id: S1, message: S2, span: MultiSpan) -> Self {
        CompilerError {
            id: id.to_string(),
            message: message.to_string(),
            span
        }
    }
}

/// The error manager
#[derive(Clone)]
pub struct ErrorManager<'tcx> {
    codemap: &'tcx CodeMap,
    error_contexts: HashMap<String, (Span, ErrorCtxt)>,
}

impl<'tcx> ErrorManager<'tcx> {
    pub fn new(codemap: &'tcx CodeMap) -> Self {
        ErrorManager {
            codemap,
            error_contexts: HashMap::new(),
        }
    }

    pub fn register(&mut self, span: Span, error_ctx: ErrorCtxt) -> Position {
        let pos_id = Uuid::new_v4().to_hyphenated().to_string();
        self.error_contexts.insert(pos_id.to_string(), (span, error_ctx));
        let lines_info = self.codemap.span_to_lines(span.source_callsite()).unwrap();
        let first_line_info = lines_info.lines.get(0).unwrap();
        let line = first_line_info.line_index as i32 + 1;
        let column = first_line_info.start_col.0 as i32 + 1;
        let pos = Position::new(line, column, pos_id.to_string());
        debug!("Register position: {:?}", pos);
        pos
    }

    pub fn translate(&self, ver_error: &VerificationError) -> CompilerError {
        let opt_error_ctx = self.error_contexts.get(&ver_error.pos_id);

        let (error_span, error_ctx) = if let Some(x) = opt_error_ctx {
            x
        } else {
            error!("Unregistered verification error: {:?}", ver_error);
            return CompilerError::new(
                ver_error.full_id.clone(),
                format!("Unregistered verification error: {}", ver_error.message),
                MultiSpan::new()
            )
        };

        match (ver_error.full_id.as_str(), error_ctx) {
            ("assert.failed:assertion.false", ErrorCtxt::Panic(PanicCause::Unknown)) => CompilerError::new(
                "P0001",
                "statement might panic",
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::Panic(PanicCause::Panic)) => CompilerError::new(
                "P0002",
                "panic!(..) statement might panic",
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::Panic(PanicCause::Assert)) => CompilerError::new(
                "P0003",
                "assert!(..) statement might not hold",
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::Panic(PanicCause::Unreachable)) => CompilerError::new(
                "P0004",
                "unreachable!(..) statement might be reachable",
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::AssertTerminator(ref message)) => CompilerError::new(
                "P0005",
                format!("assertion might fail with \"{}\"", message),
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::Panic(PanicCause::Unimplemented)) => CompilerError::new(
                "P0006",
                "unimplemented!(..) statement might be reachable",
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::AbortTerminator) => CompilerError::new(
                "P????",
                format!("statement might abort"),
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::UnreachableTerminator) => CompilerError::new(
                "P????",
                format!("unreachable code might be reachable. This might be a bug in the compiler."),
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::ExhalePostcondition) => CompilerError::new(
                "P????",
                format!("Postcondition might not hold."),
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::ExhaleLoopInvariant) => CompilerError::new(
                "P????",
                format!("The loop invariant might not hold."),
                MultiSpan::from_span(*error_span)
            ),

            ("assert.failed:assertion.false", ErrorCtxt::AssertLoopInvariant) => CompilerError::new(
                "P????",
                format!("The loop invariant might not hold."),
                MultiSpan::from_span(*error_span)
            ),

            (full_err_id, ErrorCtxt::Unexpected) => CompilerError::new(
                full_err_id,
                format!("unexpected verification error ({})", ver_error.message),
                MultiSpan::from_span(*error_span)
            ),

            (full_err_id, _) => {
                error!("Unhandled verification error: {:?}, context: {:?}", ver_error, error_ctx);
                CompilerError::new(
                    full_err_id,
                    format!("Unhandled verification error ({})", ver_error.message),
                    MultiSpan::from_span(*error_span)
                )
            },
        }
    }
}