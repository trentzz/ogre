/// Controls the level of output produced by ogre commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    /// Suppress non-essential output (only errors and requested data).
    Quiet,
    /// Default output level.
    Normal,
    /// Extra detail (instruction counts, timing, per-file info).
    Verbose,
}

impl Verbosity {
    pub fn is_quiet(self) -> bool {
        self == Verbosity::Quiet
    }

    pub fn is_verbose(self) -> bool {
        self == Verbosity::Verbose
    }
}
