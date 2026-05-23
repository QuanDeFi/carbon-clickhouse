use std::{
    sync::{LazyLock, Once},
    time::Duration,
};

use crate::metrics::{Counter, Gauge, Histogram, MetricsRegistry};

macro_rules! clickhouse_metric_family {
    (
        $prefix:literal,
        $singular:literal,
        inserted: $inserted:ident,
        failed: $failed:ident,
        inserted_bytes: $inserted_bytes:ident,
        failed_bytes: $failed_bytes:ident,
        retries: $retries:ident,
        backpressure_rejected: $backpressure_rejected:ident,
        buffered_rows: $buffered_rows:ident,
        buffered_bytes: $buffered_bytes:ident,
        active_buffers: $active_buffers:ident,
        flush_batches: $flush_batches:ident,
        flush_failed_batches: $flush_failed_batches:ident,
        flush_duration_millis: $flush_duration_millis:ident
    ) => {
        static $inserted: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".inserted"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " rows successfully inserted"
            ),
        );
        static $failed: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".failed"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " rows in failed batches"
            ),
        );
        static $inserted_bytes: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".inserted_bytes"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " bytes successfully inserted"
            ),
        );
        static $failed_bytes: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".failed_bytes"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " bytes in failed batches"
            ),
        );
        static $retries: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".retries"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " flush retry attempts"
            ),
        );
        static $backpressure_rejected: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".backpressure_rejected"),
            concat!(
                "Total number of ClickHouse ",
                $singular,
                " rows rejected by local backpressure"
            ),
        );
        static $buffered_rows: Gauge = Gauge::new(
            concat!("clickhouse.", $prefix, ".buffered_rows"),
            concat!(
                "Current number of ",
                $singular,
                " rows buffered in the ClickHouse sink"
            ),
        );
        static $buffered_bytes: Gauge = Gauge::new(
            concat!("clickhouse.", $prefix, ".buffered_bytes"),
            concat!(
                "Current number of ",
                $singular,
                " bytes buffered in the ClickHouse sink"
            ),
        );
        static $active_buffers: Gauge = Gauge::new(
            concat!("clickhouse.", $prefix, ".active_buffers"),
            concat!(
                "Current number of active ",
                $singular,
                " ClickHouse buffers"
            ),
        );
        static $flush_batches: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".flush.batches"),
            concat!(
                "Total number of successful ClickHouse ",
                $singular,
                " flush batches"
            ),
        );
        static $flush_failed_batches: Counter = Counter::new(
            concat!("clickhouse.", $prefix, ".flush.failed_batches"),
            concat!(
                "Total number of failed ClickHouse ",
                $singular,
                " flush batches"
            ),
        );
        static $flush_duration_millis: LazyLock<Histogram> = LazyLock::new(|| {
            Histogram::new(
                concat!("clickhouse.", $prefix, ".flush.duration_milliseconds"),
                concat!(
                    "Duration of ClickHouse ",
                    $singular,
                    " flush operations in milliseconds"
                ),
                vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
            )
        });
    };
}

clickhouse_metric_family!(
    "instructions",
    "instruction",
    inserted: CLICKHOUSE_INSTRUCTIONS_INSERTED,
    failed: CLICKHOUSE_INSTRUCTIONS_FAILED,
    inserted_bytes: CLICKHOUSE_INSTRUCTIONS_INSERTED_BYTES,
    failed_bytes: CLICKHOUSE_INSTRUCTIONS_FAILED_BYTES,
    retries: CLICKHOUSE_INSTRUCTIONS_RETRIES,
    backpressure_rejected: CLICKHOUSE_INSTRUCTIONS_BACKPRESSURE_REJECTED,
    buffered_rows: CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS,
    buffered_bytes: CLICKHOUSE_INSTRUCTIONS_BUFFERED_BYTES,
    active_buffers: CLICKHOUSE_INSTRUCTIONS_ACTIVE_BUFFERS,
    flush_batches: CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES,
    flush_failed_batches: CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES,
    flush_duration_millis: CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS
);

clickhouse_metric_family!(
    "accounts",
    "account",
    inserted: CLICKHOUSE_ACCOUNTS_INSERTED,
    failed: CLICKHOUSE_ACCOUNTS_FAILED,
    inserted_bytes: CLICKHOUSE_ACCOUNTS_INSERTED_BYTES,
    failed_bytes: CLICKHOUSE_ACCOUNTS_FAILED_BYTES,
    retries: CLICKHOUSE_ACCOUNTS_RETRIES,
    backpressure_rejected: CLICKHOUSE_ACCOUNTS_BACKPRESSURE_REJECTED,
    buffered_rows: CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS,
    buffered_bytes: CLICKHOUSE_ACCOUNTS_BUFFERED_BYTES,
    active_buffers: CLICKHOUSE_ACCOUNTS_ACTIVE_BUFFERS,
    flush_batches: CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES,
    flush_failed_batches: CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES,
    flush_duration_millis: CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS
);

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

impl ClickHouseMetricSet {
    fn register(self, registry: &MetricsRegistry) {
        registry.register_counter(self.inserted);
        registry.register_counter(self.failed);
        registry.register_counter(self.inserted_bytes);
        registry.register_counter(self.failed_bytes);
        registry.register_counter(self.retries);
        registry.register_counter(self.backpressure_rejected);
        registry.register_gauge(self.buffered_rows);
        registry.register_gauge(self.buffered_bytes);
        registry.register_gauge(self.active_buffers);
        registry.register_counter(self.flush_batches);
        registry.register_counter(self.flush_failed_batches);
        registry.register_histogram(self.flush_duration_millis);
    }
}

pub fn register_clickhouse_metrics() {
    REGISTER_CLICKHOUSE_METRICS.call_once(|| {
        let registry = MetricsRegistry::global();
        for family in [
            ClickHouseMetricsFamily::Instructions,
            ClickHouseMetricsFamily::Accounts,
        ] {
            family.metrics().register(registry);
        }
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
