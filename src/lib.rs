/// Fieldnote - A CLI tool for p2p sync and share
///
/// This library provides the core functionality for managing users, devices,
/// and mirroring notes between devices using the iroh p2p network.
pub mod core;
pub mod platform;
pub mod ui;

#[cfg(feature = "cli")]
pub mod cli;
