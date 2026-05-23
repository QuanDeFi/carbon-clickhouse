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

static CLICKHOUSE_INSTRUCTIONS_INSERTED_BYTES: Counter = Counter::new(
    "clickhouse.instructions.inserted_bytes",
    "Total number of ClickHouse instruction bytes successfully inserted",
);

static CLICKHOUSE_INSTRUCTIONS_FAILED_BYTES: Counter = Counter::new(
    "clickhouse.instructions.failed_bytes",
    "Total number of ClickHouse instruction bytes in failed batches",
);

static CLICKHOUSE_INSTRUCTIONS_RETRIES: Counter = Counter::new(
    "clickhouse.instructions.retries",
    "Total number of ClickHouse instruction flush retry attempts",
);

static CLICKHOUSE_INSTRUCTIONS_BACKPRESSURE_REJECTED: Counter = Counter::new(
    "clickhouse.instructions.backpressure_rejected",
    "Total number of ClickHouse instruction rows rejected by local backpressure",
);

static CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.instructions.buffered_rows",
    "Current number of instruction rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_INSTRUCTIONS_BUFFERED_BYTES: Gauge = Gauge::new(
    "clickhouse.instructions.buffered_bytes",
    "Current number of instruction bytes buffered in the ClickHouse sink",
);

static CLICKHOUSE_INSTRUCTIONS_ACTIVE_BUFFERS: Gauge = Gauge::new(
    "clickhouse.instructions.active_buffers",
    "Current number of active instruction ClickHouse buffers",
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

static CLICKHOUSE_ACCOUNTS_INSERTED_BYTES: Counter = Counter::new(
    "clickhouse.accounts.inserted_bytes",
    "Total number of ClickHouse account bytes successfully inserted",
);

static CLICKHOUSE_ACCOUNTS_FAILED_BYTES: Counter = Counter::new(
    "clickhouse.accounts.failed_bytes",
    "Total number of ClickHouse account bytes in failed batches",
);

static CLICKHOUSE_ACCOUNTS_RETRIES: Counter = Counter::new(
    "clickhouse.accounts.retries",
    "Total number of ClickHouse account flush retry attempts",
);

static CLICKHOUSE_ACCOUNTS_BACKPRESSURE_REJECTED: Counter = Counter::new(
    "clickhouse.accounts.backpressure_rejected",
    "Total number of ClickHouse account rows rejected by local backpressure",
);

static CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.accounts.buffered_rows",
    "Current number of account rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_ACCOUNTS_BUFFERED_BYTES: Gauge = Gauge::new(
    "clickhouse.accounts.buffered_bytes",
    "Current number of account bytes buffered in the ClickHouse sink",
);

static CLICKHOUSE_ACCOUNTS_ACTIVE_BUFFERS: Gauge = Gauge::new(
    "clickhouse.accounts.active_buffers",
    "Current number of active account ClickHouse buffers",
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
    inserted_bytes: &'static Counter,
    failed_bytes: &'static Counter,
    retries: &'static Counter,
    backpressure_rejected: &'static Counter,
    buffered_rows: &'static Gauge,
    buffered_bytes: &'static Gauge,
    active_buffers: &'static Gauge,
    flush_batches: &'static Counter,
    flush_failed_batches: &'static Counter,
    flush_duration_millis: &'static LazyLock<Histogram>,
}

pub fn register_clickhouse_metrics() {
    REGISTER_CLICKHOUSE_METRICS.call_once(|| {
        let registry = MetricsRegistry::global();
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_INSERTED);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FAILED);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_INSERTED_BYTES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FAILED_BYTES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_RETRIES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_BACKPRESSURE_REJECTED);
        registry.register_gauge(&CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS);
        registry.register_gauge(&CLICKHOUSE_INSTRUCTIONS_BUFFERED_BYTES);
        registry.register_gauge(&CLICKHOUSE_INSTRUCTIONS_ACTIVE_BUFFERS);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_INSERTED);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FAILED);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_INSERTED_BYTES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FAILED_BYTES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_RETRIES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_BACKPRESSURE_REJECTED);
        registry.register_gauge(&CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS);
        registry.register_gauge(&CLICKHOUSE_ACCOUNTS_BUFFERED_BYTES);
        registry.register_gauge(&CLICKHOUSE_ACCOUNTS_ACTIVE_BUFFERS);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS);
    });
}

pub(crate) fn record_buffered_rows(family: ClickHouseMetricsFamily, buffered_rows: usize) {
    family.metrics().buffered_rows.set(buffered_rows as f64);
}

pub(crate) fn record_buffer_state(
    family: ClickHouseMetricsFamily,
    buffered_rows: usize,
    buffered_bytes: usize,
    active_buffers: usize,
) {
    let metrics = family.metrics();
    metrics.buffered_rows.set(buffered_rows as f64);
    metrics.buffered_bytes.set(buffered_bytes as f64);
    metrics.active_buffers.set(active_buffers as f64);
}

pub(crate) fn record_successful_flush(
    family: ClickHouseMetricsFamily,
    rows: usize,
    bytes: usize,
    buffered_rows: usize,
    buffered_bytes: usize,
    active_buffers: usize,
    duration: Duration,
) {
    let metrics = family.metrics();
    if rows > 0 {
        metrics.inserted.inc_by(rows as u64);
        metrics.inserted_bytes.inc_by(bytes as u64);
        metrics.flush_batches.inc();
        metrics
            .flush_duration_millis
            .record(duration.as_millis() as f64);
    }
    metrics.buffered_rows.set(buffered_rows as f64);
    metrics.buffered_bytes.set(buffered_bytes as f64);
    metrics.active_buffers.set(active_buffers as f64);
}

pub(crate) fn record_failed_flush(
    family: ClickHouseMetricsFamily,
    failed_rows: usize,
    failed_bytes: usize,
    buffered_rows: usize,
    buffered_bytes: usize,
    active_buffers: usize,
) {
    let metrics = family.metrics();
    metrics.failed.inc_by(failed_rows as u64);
    metrics.failed_bytes.inc_by(failed_bytes as u64);
    metrics.flush_failed_batches.inc();
    metrics.buffered_rows.set(buffered_rows as f64);
    metrics.buffered_bytes.set(buffered_bytes as f64);
    metrics.active_buffers.set(active_buffers as f64);
}

pub(crate) fn record_retry(family: ClickHouseMetricsFamily) {
    family.metrics().retries.inc();
}

pub(crate) fn record_backpressure_rejected(family: ClickHouseMetricsFamily) {
    family.metrics().backpressure_rejected.inc();
}

impl ClickHouseMetricsFamily {
    fn metrics(self) -> ClickHouseMetricSet {
        match self {
            Self::Instructions => ClickHouseMetricSet {
                inserted: &CLICKHOUSE_INSTRUCTIONS_INSERTED,
                failed: &CLICKHOUSE_INSTRUCTIONS_FAILED,
                inserted_bytes: &CLICKHOUSE_INSTRUCTIONS_INSERTED_BYTES,
                failed_bytes: &CLICKHOUSE_INSTRUCTIONS_FAILED_BYTES,
                retries: &CLICKHOUSE_INSTRUCTIONS_RETRIES,
                backpressure_rejected: &CLICKHOUSE_INSTRUCTIONS_BACKPRESSURE_REJECTED,
                buffered_rows: &CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS,
                buffered_bytes: &CLICKHOUSE_INSTRUCTIONS_BUFFERED_BYTES,
                active_buffers: &CLICKHOUSE_INSTRUCTIONS_ACTIVE_BUFFERS,
                flush_batches: &CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES,
                flush_failed_batches: &CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES,
                flush_duration_millis: &CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS,
            },
            Self::Accounts => ClickHouseMetricSet {
                inserted: &CLICKHOUSE_ACCOUNTS_INSERTED,
                failed: &CLICKHOUSE_ACCOUNTS_FAILED,
                inserted_bytes: &CLICKHOUSE_ACCOUNTS_INSERTED_BYTES,
                failed_bytes: &CLICKHOUSE_ACCOUNTS_FAILED_BYTES,
                retries: &CLICKHOUSE_ACCOUNTS_RETRIES,
                backpressure_rejected: &CLICKHOUSE_ACCOUNTS_BACKPRESSURE_REJECTED,
                buffered_rows: &CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS,
                buffered_bytes: &CLICKHOUSE_ACCOUNTS_BUFFERED_BYTES,
                active_buffers: &CLICKHOUSE_ACCOUNTS_ACTIVE_BUFFERS,
                flush_batches: &CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES,
                flush_failed_batches: &CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES,
                flush_duration_millis: &CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS,
            },
        }
    }
}
