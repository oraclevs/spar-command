//! Neutral, dependency-free command/execution-plan data model shared by
//! the `spar` compiler (which builds plans from Spar source) and
//! `spar-process` (which executes them). See the design doc:
//! docs/superpowers/specs/2026-09-15-spar-command-crate-design.md
//! (workspace-level `docs/superpowers/`, not part of this repo).

pub mod plan;

pub use plan::*;
