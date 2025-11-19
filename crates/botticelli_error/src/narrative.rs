//! Narrative error types.

/// Specific error conditions for narrative operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NarrativeErrorKind {
    /// Failed to read narrative file
    FileRead(String),
    /// Failed to parse TOML content
    TomlParse(String),
    /// Table of contents is empty
    EmptyToc,
    /// Act referenced in table of contents does not exist in acts map
    MissingAct(String),
    /// Act prompt is empty or contains only whitespace
    EmptyPrompt(String),
    /// Template field required but not set
    MissingTemplate,
    /// Failed to assemble prompt with schema injection
    PromptAssembly {
        /// Act name
        act: String,
        /// Error message
        message: String,
    },
}

impl std::fmt::Display for NarrativeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NarrativeErrorKind::FileRead(msg) => {
                write!(f, "Failed to read narrative file: {}", msg)
            }
            NarrativeErrorKind::TomlParse(msg) => write!(f, "Failed to parse TOML: {}", msg),
            NarrativeErrorKind::EmptyToc => {
                write!(f, "Table of contents (toc.order) cannot be empty")
            }
            NarrativeErrorKind::MissingAct(act) => write!(
                f,
                "Act '{}' referenced in toc.order does not exist in acts map",
                act
            ),
            NarrativeErrorKind::EmptyPrompt(act) => write!(f, "Act '{}' has an empty prompt", act),
            NarrativeErrorKind::MissingTemplate => {
                write!(f, "Template field is required for prompt assembly")
            }
            NarrativeErrorKind::PromptAssembly { act, message } => write!(
                f,
                "Failed to assemble prompt for act '{}': {}",
                act, message
            ),
        }
    }
}

/// Error type for narrative operations.
///
/// # Examples
///
/// ```
/// use botticelli_error::{NarrativeError, NarrativeErrorKind};
///
/// let err = NarrativeError::new(NarrativeErrorKind::EmptyToc);
/// assert!(format!("{}", err).contains("empty"));
/// ```
#[derive(Debug, Clone)]
pub struct NarrativeError {
    /// The specific error condition
    pub kind: NarrativeErrorKind,
    /// Line number where the error occurred
    pub line: u32,
    /// Source file where the error occurred
    pub file: &'static str,
}

impl NarrativeError {
    /// Create a new NarrativeError with automatic location tracking.
    #[track_caller]
    pub fn new(kind: NarrativeErrorKind) -> Self {
        let location = std::panic::Location::caller();
        Self {
            kind,
            line: location.line(),
            file: location.file(),
        }
    }
}

impl std::fmt::Display for NarrativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Narrative Error: {} at line {} in {}",
            self.kind, self.line, self.file
        )
    }
}

impl std::error::Error for NarrativeError {}
