use std::{
    sync::{LazyLock, Once},
    time::Duration,
};

use crate::metrics::{Counter, Gauge, Histogram, MetricsRegistry};

static CLICKHOUSE_INSTRUCTIONS_INSERTED: Counter = Counter::new(
    "clickhouse.instructions.inserted",
    "Total number of ClickHouse instruction rows successfully inserted",
);

static CLICKHOUSE_INSTRUCTIONS_FAILED: Counter = Counter::new(
    "clickhouse.instructions.failed",
    "Total number of ClickHouse instruction rows in failed batches",
);

static CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.instructions.buffered_rows",
    "Current number of instruction rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES: Counter = Counter::new(
    "clickhouse.instructions.flush.batches",
    "Total number of successful ClickHouse instruction flush batches",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES: Counter = Counter::new(
    "clickhouse.instructions.flush.failed_batches",
    "Total number of failed ClickHouse instruction flush batches",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::new(
        "clickhouse.instructions.flush.duration_milliseconds",
        "Duration of ClickHouse instruction flush operations in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
    )
});

static CLICKHOUSE_ACCOUNTS_INSERTED: Counter = Counter::new(
    "clickhouse.accounts.inserted",
    "Total number of ClickHouse account rows successfully inserted",
);

static CLICKHOUSE_ACCOUNTS_FAILED: Counter = Counter::new(
    "clickhouse.accounts.failed",
    "Total number of ClickHouse account rows in failed batches",
);

static CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.accounts.buffered_rows",
    "Current number of account rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES: Counter = Counter::new(
    "clickhouse.accounts.flush.batches",
    "Total number of successful ClickHouse account flush batches",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES: Counter = Counter::new(
    "clickhouse.accounts.flush.failed_batches",
    "Total number of failed ClickHouse account flush batches",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::new(
        "clickhouse.accounts.flush.duration_milliseconds",
        "Duration of ClickHouse account flush operations in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
    )
});

static REGISTER_CLICKHOUSE_METRICS: Once = Once::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClickHouseMetricsFamily {
    Instructions,
    Accounts,
}

#[derive(Clone, Copy)]
struct ClickHouseMetricSet {
    inserted: &'static Counter,
    failed: &'static Counter,
    buffered_rows: &'static Gauge,
    flush_batches: &'static Counter,
    flush_failed_batches: &'static Counter,
    flush_duration_millis: &'static LazyLock<Histogram>,
}

pub fn register_clickhouse_metrics() {
    REGISTER_CLICKHOUSE_METRICS.call_once(|| {
        let registry = MetricsRegistry::global();
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_INSERTED);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FAILED);
        registry.register_gauge(&CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_INSERTED);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FAILED);
        registry.register_gauge(&CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS);
    });
}

pub(crate) fn record_buffered_rows(family: ClickHouseMetricsFamily, buffered_rows: usize) {
    family.metrics().buffered_rows.set(buffered_rows as f64);
}

pub(crate) fn record_successful_flush(
    family: ClickHouseMetricsFamily,
    rows: usize,
    buffered_rows: usize,
    duration: Duration,
) {
    let metrics = family.metrics();
    if rows > 0 {
        metrics.inserted.inc_by(rows as u64);
        metrics.flush_batches.inc();
        metrics
            .flush_duration_millis
            .record(duration.as_millis() as f64);
    }
    metrics.buffered_rows.set(buffered_rows as f64);
}

pub(crate) fn record_failed_flush(
    family: ClickHouseMetricsFamily,
    failed_rows: usize,
    buffered_rows: usize,
) {
    let metrics = family.metrics();
    metrics.failed.inc_by(failed_rows as u64);
    metrics.flush_failed_batches.inc();
    metrics.buffered_rows.set(buffered_rows as f64);
}

impl ClickHouseMetricsFamily {
    fn metrics(self) -> ClickHouseMetricSet {
        match self {
            Self::Instructions => ClickHouseMetricSet {
                inserted: &CLICKHOUSE_INSTRUCTIONS_INSERTED,
                failed: &CLICKHOUSE_INSTRUCTIONS_FAILED,
                buffered_rows: &CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS,
                flush_batches: &CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES,
                flush_failed_batches: &CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES,
                flush_duration_millis: &CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS,
            },
            Self::Accounts => ClickHouseMetricSet {
                inserted: &CLICKHOUSE_ACCOUNTS_INSERTED,
                failed: &CLICKHOUSE_ACCOUNTS_FAILED,
                buffered_rows: &CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS,
                flush_batches: &CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES,
                flush_failed_batches: &CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES,
                flush_duration_millis: &CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS,
            },
        }
    }
}
