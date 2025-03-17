pub mod entrypoint;
pub mod state;
pub mod processor;
pub mod error;

pub use state::*;
pub use processor::*;
pub use error::*;
pub use entrypoint::process_instruction;  // Explicitly make process_instruction public