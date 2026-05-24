use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::clickhouse::{http::ClickHouseHttpError, ClickHouseConfig};

pub(crate) fn should_retry_clickhouse(
    error: &ClickHouseHttpError,
    attempt: usize,
    max_retries: usize,
) -> bool {
    error.kind.is_retryable() && attempt < max_retries
}

pub(crate) fn clickhouse_retry_delay(config: &ClickHouseConfig, attempt: usize) -> Duration {
    let base = config.retry_settings.initial_backoff;
    let multiplier = 1u32.checked_shl(attempt.min(16) as u32).unwrap_or(u32::MAX);
    let mut delay = base
        .saturating_mul(multiplier)
        .min(config.retry_settings.max_backoff);

    if config.retry_settings.jitter && !delay.is_zero() {
        let jitter_nanos = delay.as_nanos().saturating_div(10).min(u64::MAX as u128) as u64;
        if jitter_nanos > 0 {
            delay = delay.saturating_add(Duration::from_nanos(pseudo_jitter_nanos(jitter_nanos)));
        }
    }

    delay.min(config.retry_settings.max_backoff)
}

fn pseudo_jitter_nanos(max: u64) -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64 % max)
        .unwrap_or_default()
}
