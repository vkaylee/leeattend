//! CompiledPolicy — pre-resolved version-scoped layers, plus an `ArcSwap`
//! cache keyed by `policy_version_id`.
//!
//! Layers cached: L1 base + L2a/L2b/L2c maps. L2.5 (employee modifier) and
//! L3 (exception) are per-employee/date and loaded on the eval path.
//!
//! Cache semantics:
//! - `PolicyVersionCache::publish(version_id, compiled)` — atomic swap,
//!   O(entries) copy of the outer HashMap under the hood.
//! - `PolicyVersionCache::get(version_id)` — lock-free read via ArcSwap.
//! - FROZEN versions never mutate ([[ADR-004]]); publish for them once, read
//!   forever.

use crate::domain::policy::cascade::base::BaseRules;
use crate::domain::policy::cascade::partial::PartialPolicyRules;
use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// A policy version compiled to in-memory form.
///
/// The three L2 maps are keyed by their scope identifier:
/// - `l2a_by_day_type`: e.g. `"weekday"`, `"sunday"`, `"holiday_national"`.
/// - `l2b_by_shift_template`: `shift_templates.id`.
/// - `l2c_by_shift_version`:  `shift_versions.id`.
#[derive(Debug, Clone)]
pub struct CompiledPolicy {
    pub policy_version_id: Uuid,
    pub base_rules: BaseRules,
    pub l2a_by_day_type: HashMap<String, PartialPolicyRules>,
    pub l2b_by_shift_template: HashMap<Uuid, PartialPolicyRules>,
    pub l2c_by_shift_version: HashMap<Uuid, PartialPolicyRules>,
}

impl CompiledPolicy {
    pub fn empty(policy_version_id: Uuid, base_rules: BaseRules) -> Self {
        Self {
            policy_version_id,
            base_rules,
            l2a_by_day_type: HashMap::new(),
            l2b_by_shift_template: HashMap::new(),
            l2c_by_shift_version: HashMap::new(),
        }
    }
}

/// Lock-free cache of `Arc<CompiledPolicy>` keyed by policy version id.
///
/// Reader path (`get`) never blocks; writer path (`publish` / `evict`) does a
/// copy-and-swap of the inner map. This is fine for our access pattern:
/// thousands of reads per publish.
pub struct PolicyVersionCache {
    inner: ArcSwap<HashMap<Uuid, Arc<CompiledPolicy>>>,
}

impl PolicyVersionCache {
    pub fn new() -> Self {
        Self {
            inner: ArcSwap::from_pointee(HashMap::new()),
        }
    }

    pub fn get(&self, policy_version_id: Uuid) -> Option<Arc<CompiledPolicy>> {
        self.inner.load().get(&policy_version_id).cloned()
    }

    /// Insert-or-replace a compiled policy. Safe to call concurrently.
    pub fn publish(&self, compiled: Arc<CompiledPolicy>) {
        let id = compiled.policy_version_id;
        self.inner.rcu(|current| {
            let mut next = (**current).clone();
            next.insert(id, compiled.clone());
            next
        });
    }

    /// Remove a version from the cache. Rare — used when a DRAFT is rewritten.
    /// FROZEN versions must never be evicted ([[ADR-004]]).
    pub fn evict(&self, policy_version_id: Uuid) {
        self.inner.rcu(|current| {
            let mut next = (**current).clone();
            next.remove(&policy_version_id);
            next
        });
    }

    pub fn len(&self) -> usize {
        self.inner.load().len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.load().is_empty()
    }
}

impl Default for PolicyVersionCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::policy::cascade::caps::BaseFieldSpec;
    use crate::models::enums::EarlyArrivalStrategy;

    fn base(default: i32) -> BaseRules {
        BaseRules {
            grace_period_minutes: BaseFieldSpec {
                default_value: default,
                absolute_min: Some(0),
                absolute_max: Some(120),
            },
            authorized_early_leave_minutes: BaseFieldSpec {
                default_value: 0,
                absolute_min: Some(0),
                absolute_max: Some(240),
            },
            is_early_leave_paid: false,
            early_arrival_strategy: EarlyArrivalStrategy::Allow,
            is_break_auto_deduct: true,
            is_break_overtime: false,
            allow_early_ot: false,
        }
    }

    #[test]
    fn empty_cache_returns_none() {
        let cache = PolicyVersionCache::new();
        assert!(cache.is_empty());
        assert!(cache.get(Uuid::new_v4()).is_none());
    }

    #[test]
    fn publish_then_get_returns_same_arc() {
        let cache = PolicyVersionCache::new();
        let id = Uuid::new_v4();
        let compiled = Arc::new(CompiledPolicy::empty(id, base(10)));
        cache.publish(compiled.clone());

        let fetched = cache.get(id).unwrap();
        assert!(Arc::ptr_eq(&compiled, &fetched));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn publish_same_id_replaces_previous() {
        let cache = PolicyVersionCache::new();
        let id = Uuid::new_v4();
        cache.publish(Arc::new(CompiledPolicy::empty(id, base(10))));
        cache.publish(Arc::new(CompiledPolicy::empty(id, base(30))));

        let latest = cache.get(id).unwrap();
        assert_eq!(latest.base_rules.grace_period_minutes.default_value, 30);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn evict_removes_only_the_target() {
        let cache = PolicyVersionCache::new();
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        cache.publish(Arc::new(CompiledPolicy::empty(a, base(10))));
        cache.publish(Arc::new(CompiledPolicy::empty(b, base(20))));
        assert_eq!(cache.len(), 2);

        cache.evict(a);
        assert!(cache.get(a).is_none());
        assert!(cache.get(b).is_some());
        assert_eq!(cache.len(), 1);
    }
}
