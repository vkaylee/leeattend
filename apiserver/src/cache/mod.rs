//! Cache Module
//!
//! Provides Redis-based caching for static data to reduce DB queries.
//! Implements cache-aside pattern with configurable TTL.

mod manager;
mod redis_cache;
mod roster_cache;
mod shift_cache;
mod workspace_cache;

pub use manager::CacheManager;
pub use redis_cache::RedisCache;
pub use roster_cache::{
    build_roster_data, RosterCache, RosterData, RosterDeptFixedShift, RosterDeptHistory,
    RosterDeptPattern, RosterFixedAssignment, RosterPatternEntry, RosterShift,
};
pub use shift_cache::{CachedShift, ShiftCache};
pub use workspace_cache::{CachedWorkspaceConfig, WorkspaceCache};

use std::time::Duration;

/// Default cache TTL: 5 minutes
/// Balance between freshness and performance
pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);
