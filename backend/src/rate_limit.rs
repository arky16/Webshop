
use redis::AsyncCommands;

use crate::state::AppState;

pub async fn is_rate_limited(state: &AppState, key: &str, max_attempts: u32, window_seconds: u64) -> Result<bool, redis::RedisError> {
    let mut connection = state.redis
        .get_multiplexed_async_connection()
        .await?;

    let attempts: u64 = connection.incr(key, 1).await?;

    if attempts == 1 {
        let _: bool = connection
            .expire(key, window_seconds as i64)
            .await?;
    }

    Ok(attempts > max_attempts as u64)
}

pub async fn is_login_rate_limited(state: &AppState, key: &str, max_attempts: u32) -> Result<bool, redis::RedisError> {
    let mut connection = state.redis
        .get_multiplexed_async_connection()
        .await?;

    let attempts: Option<u32> = connection.get(key).await?;

    Ok(attempts.unwrap_or(0) >= max_attempts)
}

pub async fn record_failed_attempt(state: &AppState, key: &str, window_seconds: u64) -> Result<(), redis::RedisError> {
    let mut connection = state.redis
        .get_multiplexed_async_connection()
        .await?;

    let attempts: u64 = connection.incr(key, 1).await?;

    if attempts == 1 {
        let _: bool = connection
            .expire(key, window_seconds as i64)
            .await?;
    }

    Ok(())
}

pub async fn reset_rate_limit(state: &AppState, key: &str) -> Result<(), redis::RedisError> {
    let mut connection = state.redis
        .get_multiplexed_async_connection()
        .await?;

    let _: u64 = connection.del(key).await?;

    Ok(())
}

pub fn rate_limit_key(ip: &str, action: &str) -> String {
    format!("rate_limit:{action}:{ip}")
}

