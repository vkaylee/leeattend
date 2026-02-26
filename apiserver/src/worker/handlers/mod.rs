//! Job handlers for different task types.

pub mod attendance;
pub mod merge;

pub use attendance::AttendanceHandler;
pub use merge::MergeHandler;
