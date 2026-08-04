pub mod context;
pub mod context_helpers;
pub mod service;
pub mod service_helpers;
pub mod service_propagation;
pub mod service_range;
pub mod service_single;
pub mod strategy;
pub mod strategy_split;
pub mod strategy_window;
pub mod types;

#[cfg(test)]
pub mod tests;

pub use service::PairingService;
pub use strategy::{get_strategy, ShiftAwareStrategy};
pub use types::{
    BreakInterval, EntryType, LogEvent, NewPairedSessionParams, PairedSession, PairingContext,
    PairingMetadata, PairingNote, PairingProcessor, PairingServiceArgs, PairingShiftConfig,
};
