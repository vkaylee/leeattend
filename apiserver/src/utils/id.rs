/// Centralized ID generation using UUID v7 (time-sorted).
///
/// UUID v7 embeds a millisecond-precision timestamp in the most significant
/// bits, producing monotonically increasing values that are friendly to
/// B-Tree indexes. This eliminates the page-split fragmentation caused by
/// random UUID v4 on high-write tables like `raw_attendance_logs`,
/// `time_entries`, and `audit_logs`.
///
/// # Usage
/// ```rust
/// use crate::utils::id::new_id;
/// let id = new_id(); // time-sorted UUID v7
/// ```
///
/// # When to use `new_id()` vs `Uuid::new_v4()`
/// - **Production records** (DB rows): Always use `new_id()` → v7
/// - **Test fixtures**: `Uuid::new_v4()` is fine — ordering doesn't matter
/// - **Ephemeral tokens** (refresh tokens, nonces): `Uuid::new_v4()` is fine
pub fn new_id() -> uuid::Uuid {
    uuid::Uuid::now_v7()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_id_is_v7() {
        let id = new_id();
        assert_eq!(id.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn test_new_id_is_monotonically_increasing() {
        let id1 = new_id();
        let id2 = new_id();
        // v7 UUIDs generated sequentially should be ordered
        assert!(id2 > id1, "id2 ({id2}) should be greater than id1 ({id1})");
    }

    #[test]
    fn test_new_id_uniqueness() {
        let ids: Vec<uuid::Uuid> = (0..1000).map(|_| new_id()).collect();
        let unique: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(unique.len(), 1000);
    }

    #[test]
    fn test_new_id_string_format() {
        let id = new_id();
        let s = id.to_string();
        // Standard UUID format: 8-4-4-4-12
        assert_eq!(s.len(), 36);
        assert_eq!(s.chars().filter(|c| *c == '-').count(), 4);
    }
}
