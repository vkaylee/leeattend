//! Job handlers for different report types.

pub mod attendance;
pub mod merge;

pub use attendance::AttendanceHandler;
pub use merge::MergeHandler;
