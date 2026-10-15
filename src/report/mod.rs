//! `src/report`: `status` human & JSON, verbose reasons, heartbeat text, colour.
//!
//! Composes only (src:C): each part lives in its own submodule and the
//! public surface is re-exported here and from the crate root (src:V3).
//!
//! Every renderer writes to an [`std::io::Write`] it is given, so tests
//! write to a `Vec<u8>` and only the caller (`src/cli`) owns the real
//! streams; nothing here reads the environment or asks for a TTY.

mod colour;
mod json;
mod model;
mod route;
mod table;

pub use colour::{Paint, Style};
pub use json::{SCHEMA_VERSION, json_string, write_json};
pub use model::{StatusReport, StatusRow, switch_name};
pub use route::{Output, Stream, Streams};
pub use table::write_status;
