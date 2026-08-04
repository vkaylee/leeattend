use crate::errors::AppError;
use crate::models::User;
use crate::repositories::user_repository::UserRepository;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use uuid::Uuid;

/// A generic batch loader utility to resolve cross-database N+1 queries.
///
/// It takes a list of keys and a batch load function, executing a single batch request
/// instead of individual requests, preventing N+1 queries.
pub struct BatchLoader;

impl BatchLoader {
    /// Loads values for the given keys in a batch using the provided load function.
    ///
    /// Deduplicates keys to avoid redundant querying.
    pub async fn load<K, V, F, Fut>(keys: &[K], load_fn: F) -> Result<HashMap<K, V>, AppError>
    where
        K: Eq + Hash + Clone + Send + Sync + 'static,
        V: Send + 'static,
        F: FnOnce(Vec<K>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<HashMap<K, V>, AppError>> + Send,
    {
        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        // Deduplicate keys
        let mut unique_keys = Vec::new();
        for key in keys {
            if !unique_keys.contains(key) {
                unique_keys.push(key.clone());
            }
        }

        load_fn(unique_keys).await
    }

    /// Specifically loads a map of Uuid to User from the System DB in a single batch query.
    ///
    /// This resolves the common cross-database join pattern where we have employee rows in a Tenant DB
    /// and need to fetch corresponding user profile data from the System DB.
    pub async fn load_users(
        user_ids: &[Uuid],
        user_repo: Arc<dyn UserRepository>,
    ) -> Result<HashMap<Uuid, User>, AppError> {
        Self::load(user_ids, move |ids| async move {
            let users = user_repo.get_users_by_ids(&ids).await?;
            let mut map = HashMap::new();
            for user in users {
                map.insert(user.id(), user);
            }
            Ok(map)
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;
    use crate::repositories::user_repository::MockUserRepository;

    #[tokio::test]
    async fn test_generic_batch_loader() {
        let keys = vec![1, 2, 2, 3, 1];

        let result = BatchLoader::load(&keys, |unique_keys| async move {
            assert_eq!(unique_keys.len(), 3);
            assert!(unique_keys.contains(&1));
            assert!(unique_keys.contains(&2));
            assert!(unique_keys.contains(&3));

            let mut map = HashMap::new();
            for k in unique_keys {
                map.insert(k, k * 10);
            }
            Ok(map)
        })
        .await
        .unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&1), Some(&10));
        assert_eq!(result.get(&2), Some(&20));
        assert_eq!(result.get(&3), Some(&30));
    }

    #[tokio::test]
    async fn test_load_users_empty() {
        let mock_repo = Arc::new(MockUserRepository::new());
        let result = BatchLoader::load_users(&[], mock_repo).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_load_users_with_mock() {
        let mut mock_repo = MockUserRepository::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let user1 = User::new(NewUserParams {
            id: id1,
            email: "user1@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: chrono::Utc::now(),
            full_name: Some("User One".to_string()),
            avatar_url: None,
            bio: None,
            preferred_language: None,
            avatar_file_id: None,
        });

        let user2 = User::new(NewUserParams {
            id: id2,
            email: "user2@example.com".to_string(),
            password_hash: "hash".to_string(),
            created_at: chrono::Utc::now(),
            full_name: Some("User Two".to_string()),
            avatar_url: None,
            bio: None,
            preferred_language: None,
            avatar_file_id: None,
        });

        let expected_users = vec![user1.clone(), user2.clone()];

        mock_repo
            .expect_get_users_by_ids()
            .withf(move |ids| ids.contains(&id1) && ids.contains(&id2))
            .times(1)
            .returning(move |_| Ok(expected_users.clone()));

        let result = BatchLoader::load_users(&[id1, id2, id1], Arc::new(mock_repo))
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&id1).unwrap().email(), "user1@example.com");
        assert_eq!(result.get(&id2).unwrap().email(), "user2@example.com");
    }
}
