use redis::{AsyncCommands, Client};
use std::sync::Arc;

#[derive(Clone)]
pub struct Cache {
    pub client: Arc<Client>,
}

impl Cache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }

    pub async fn get_cached_tasks(&self, user_id: &str) -> Option<String> {
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to redis: {}", e);
                return None;
            }
        };

        let key = format!("user:{}:tasks", user_id);
        let result: redis::RedisResult<Option<String>> = con.get(key).await;

        match result {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Redis get error: {}", e);
                None
            }
        }
    }

    pub async fn set_cached_tasks(&self, user_id: &str, tasks_json: &str) {
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to redis: {}", e);
                return;
            }
        };

        let key = format!("user:{}:tasks", user_id);
        // Expiration of 1 hour for cache
        let _: redis::RedisResult<()> = con.set_ex(key, tasks_json, 3600).await;
    }

    pub async fn invalidate_user_tasks(&self, user_id: &str) {
        let mut con = match self.client.get_multiplexed_async_connection().await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to redis: {}", e);
                return;
            }
        };

        let key = format!("user:{}:tasks", user_id);
        let _: redis::RedisResult<()> = con.del(key).await;
    }
}
