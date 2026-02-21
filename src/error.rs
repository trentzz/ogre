use thiserror::Error;

#[derive(Error, Debug)]
pub enum OgreError {
    #[error("unmatched `]`")]
    UnmatchedCloseBracket,

    #[error("unmatched `[` at op index {0}")]
    UnmatchedOpenBracket(usize),

    #[error("bracket mismatch: {0}")]
    BracketMismatch(String),

    #[error("data pointer out of bounds ({0})")]
    TapeOverflow(String),

    #[error("cycle detected: {0}")]
    CycleDetected(String),

    #[error("import cycle detected: {0}")]
    ImportCycle(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("unknown function: {0}")]
    UnknownFunction(String),

    #[error("unknown directive: @{0}")]
    UnknownDirective(String),

    #[error("unknown standard library module: {0}")]
    UnknownStdModule(String),

    #[error("no C compiler found. Install gcc, clang, or ensure 'cc' is available on PATH")]
    CompilerNotFound,

    #[error("compilation failed: {0}")]
    CompilationFailed(String),

    #[error("invalid project: {0}")]
    InvalidProject(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("timeout: instruction limit of {0} reached")]
    Timeout(u64),

    #[error("{0}")]
    Other(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type OgreResult<T> = Result<T, OgreError>;
