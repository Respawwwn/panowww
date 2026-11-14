//! Why a Stitch stops (spec §16.5).

use panowww_core::ports::StoreError;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StitchError {
    /// Too few Shots could be linked to measure the field of view and
    /// place the sphere.
    #[error("Too few Shots could be linked: {linked} of {total}.")]
    TooFewLinked { linked: usize, total: usize },
    /// The cancel flag was raised; nothing was produced.
    #[error("Stopped.")]
    Cancelled,
    /// A Shot could not be read.
    #[error("Shot r{row:02}_c{col:02} could not be read: {source}")]
    Shot {
        row: u32,
        col: u32,
        #[source]
        source: StoreError,
    },
    /// The Shots do not form the grid the Session describes.
    #[error("{0}")]
    Grid(String),
    /// The Stitch would exceed the memory the caller allows.
    #[error("The Stitch needs about {needed_mb} MB, more than the {budget_mb} MB allowed.")]
    Memory { needed_mb: u64, budget_mb: u64 },
    /// A stage that cannot fail by design did, or is not written yet.
    #[error("Internal: {0}")]
    Internal(&'static str),
}
