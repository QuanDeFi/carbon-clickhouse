use {
    async_trait::async_trait,
    carbon_core::error::{CarbonResult, Error as CarbonError},
    carbon_core::metrics::Metrics,
    carbon_core::postgres::processors::PostgresInstructionProcessor,
    carbon_jupiter_swap_decoder::{
        instructions::{
            postgres::{JupiterSwapInstructionWithMetadata, JupiterSwapInstructionsMigration},
            JupiterSwapInstruction,
        },
        JupiterSwapDecoder, PROGRAM_ID as JUPITER_SWAP_PROGRAM_ID,
    },
    carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler},
    carbon_rpc_transaction_crawler_datasource::{ConnectionConfig, Filters, RpcTransactionCrawler},
    chrono::Utc,
    simplelog::{
        ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
        TerminalMode, WriteLogger,
    },
    solana_client::nonblocking::rpc_client::RpcClient,
    solana_transaction_status::UiTransactionEncoding,
    sqlx::postgres::PgPoolOptions,
    sqlx_migrator::{Info, Migrate, Migrator, Plan},
    std::{
        collections::{HashMap, HashSet, VecDeque},
        env::{self, VarError},
        fmt::Display,
        fs::{self, File},
        path::PathBuf,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc, Mutex, MutexGuard,
        },
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    },
};

const SLOT_STAGE_COUNT: usize = LOGICAL_STAGE_METRICS.len();
const SLOT_STAGE_FALLBACK_TTL: Duration = Duration::from_secs(30);
const DEFAULT_STAGE_LOG_INTERVAL_MS: u64 = 2_000;
const DEFAULT_STAGE_HOUSEKEEPING_INTERVAL_MS: u64 = 250;
const DEFAULT_POSTGRES_STATEMENT_TIMEOUT_MS: u64 = 10_000;
const DEFAULT_POSTGRES_LOCK_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_POSTGRES_ACQUIRE_TIMEOUT_MS: u64 = 5_000;
const MAX_SLOTS_PER_STAGE_LOG: usize = 12;
const LOGICAL_STAGE_METRICS: [(&str, &str); 11] = [
    ("stage.schedule_slot.slot", "schedule_slot"),
    ("stage.fetch_block_get_block.slot", "fetch_block"),
    (
        "stage.extract_transactions_from_block.slot",
        "extract_transactions",
    ),
    (
        "stage.build_transaction_update.slot",
        "build_transaction_updates",
    ),
    (
        "stage.enqueue_pipeline_update.slot",
        "enqueue_pipeline_updates",
    ),
    (
        "stage.build_core_transaction_metadata.slot",
        "build_instructions",
    ),
    (
        "stage.apply_instruction_filters.slot",
        "filter_instructions",
    ),
    (
        "stage.decode_instruction_jupiter_swap_decoder.slot",
        "decode_jupiter_swaps",
    ),
    (
        "stage.process_instruction_postgres_instruction_processor.slot",
        "process_jupiter_swaps",
    ),
    (
        "stage.build_instruction_row_wrapper.slot",
        "build_postgres_rows",
    ),
    (
        "stage.upsert_instruction_row_postgres.slot",
        "write_postgres_rows",
    ),
];

struct SlotStageState {
    stage_active_slots: Vec<VecDeque<u64>>,
    stage_slot_activity: Vec<HashMap<u64, u64>>,
    stage_slot_last_touched: Vec<HashMap<u64, Instant>>,
    stage_slot_aggregates: Vec<HashMap<u64, StageAggregate>>,
    stage_last_completed_slot: Vec<Option<u64>>,
    stage_window_stats: Vec<StageWindowStats>,
    sealed_slots: HashSet<u64>,
}

struct StageAggregate {
    started_at: Instant,
    counters: HashMap<&'static str, u64>,
}

#[derive(Clone, Default)]
struct StageWindowStats {
    entered: u64,
    completed: u64,
    counters: HashMap<&'static str, u64>,
    duration_total_ms: u128,
    duration_max_ms: u128,
    last_seen_slot: Option<u64>,
    last_completed_slot: Option<u64>,
    slots: VecDeque<u64>,
}

struct ReportSnapshot {
    header: String,
    rows: Vec<String>,
}

struct StageSlotMetrics {
    run_started: AtomicBool,
    slot_state: Mutex<SlotStageState>,
    counters: Mutex<HashMap<String, u64>>,
    report_counter_snapshot: Mutex<HashMap<String, u64>>,
    histograms_last: Mutex<HashMap<String, f64>>,
    gauges: Mutex<HashMap<String, f64>>,
}

impl StageSlotMetrics {
    fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn new() -> Self {
        Self {
            run_started: AtomicBool::new(false),
            slot_state: Mutex::new(SlotStageState {
                stage_active_slots: (0..SLOT_STAGE_COUNT).map(|_| VecDeque::new()).collect(),
                stage_slot_activity: (0..SLOT_STAGE_COUNT).map(|_| HashMap::new()).collect(),
                stage_slot_last_touched: (0..SLOT_STAGE_COUNT).map(|_| HashMap::new()).collect(),
                stage_slot_aggregates: (0..SLOT_STAGE_COUNT).map(|_| HashMap::new()).collect(),
                stage_last_completed_slot: (0..SLOT_STAGE_COUNT).map(|_| None).collect(),
                stage_window_stats: (0..SLOT_STAGE_COUNT)
                    .map(|_| StageWindowStats::default())
                    .collect(),
                sealed_slots: HashSet::new(),
            }),
            counters: Mutex::new(HashMap::new()),
            report_counter_snapshot: Mutex::new(HashMap::new()),
            histograms_last: Mutex::new(HashMap::new()),
            gauges: Mutex::new(HashMap::new()),
        }
    }

    fn stage_display_name_by_index(stage_index: usize) -> &'static str {
        LOGICAL_STAGE_METRICS
            .get(stage_index)
            .map(|(_, display_name)| *display_name)
            .unwrap_or("unknown_stage")
    }

    fn stage_label_width() -> usize {
        LOGICAL_STAGE_METRICS
            .iter()
            .map(|(_, display_name)| display_name.len())
            .max()
            .unwrap_or(0)
    }

    fn ensure_stage_aggregate(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
    ) -> &mut StageAggregate {
        if let Some(stage_stats) = slot_state.stage_window_stats.get_mut(stage_index) {
            stage_stats.last_seen_slot = Some(slot);
            Self::record_window_slot(stage_stats, slot);
        }
        slot_state.stage_slot_aggregates[stage_index]
            .entry(slot)
            .or_insert_with(|| StageAggregate {
                started_at: Instant::now(),
                counters: HashMap::new(),
            })
    }

    fn bump_window_counter(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
        counter: &'static str,
        amount: u64,
    ) {
        if let Some(stage_stats) = slot_state.stage_window_stats.get_mut(stage_index) {
            *stage_stats.counters.entry(counter).or_insert(0) += amount;
            stage_stats.last_seen_slot = Some(slot);
            Self::record_window_slot(stage_stats, slot);
        }
    }

    fn record_window_slot(stage_stats: &mut StageWindowStats, slot: u64) {
        stage_stats.slots.retain(|queued_slot| *queued_slot != slot);
        stage_stats.slots.push_back(slot);
    }

    fn bump_stage_counter(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
        counter: &'static str,
    ) {
        let aggregate = Self::ensure_stage_aggregate(slot_state, stage_index, slot);
        *aggregate.counters.entry(counter).or_insert(0) += 1;
        Self::bump_window_counter(slot_state, stage_index, slot, counter, 1);
    }

    fn record_stage_completion(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
        aggregate: StageAggregate,
    ) {
        let duration_ms = aggregate.started_at.elapsed().as_millis();
        if let Some(stage_stats) = slot_state.stage_window_stats.get_mut(stage_index) {
            stage_stats.completed = stage_stats.completed.saturating_add(1);
            stage_stats.duration_total_ms =
                stage_stats.duration_total_ms.saturating_add(duration_ms);
            stage_stats.duration_max_ms = stage_stats.duration_max_ms.max(duration_ms);
            stage_stats.last_completed_slot = Some(slot);
            Self::record_window_slot(stage_stats, slot);
        }
        if let Some(last_completed) = slot_state.stage_last_completed_slot.get_mut(stage_index) {
            *last_completed = Some(slot);
        }
    }

    fn format_stage_slots(queue: &VecDeque<u64>) -> String {
        if queue.is_empty() {
            return "[-]".to_string();
        }

        let shown: Vec<String> = queue
            .iter()
            .take(MAX_SLOTS_PER_STAGE_LOG)
            .map(|slot| slot.to_string())
            .collect();
        let overflow = queue.len().saturating_sub(MAX_SLOTS_PER_STAGE_LOG);
        if overflow > 0 {
            format!("[{} ... +{} more]", shown.join(","), overflow)
        } else {
            format!("[{}]", shown.join(","))
        }
    }

    fn stage_window_slots(slot_state: &SlotStageState, stage_index: usize) -> String {
        let window_slots = slot_state
            .stage_window_stats
            .get(stage_index)
            .map(|stats| &stats.slots);

        if let Some(window_slots) = window_slots {
            if !window_slots.is_empty() {
                return Self::format_stage_slots(window_slots);
            }
        }
        "[-]".to_string()
    }

    fn stage_display_slot(slot_state: &SlotStageState, stage_index: usize) -> String {
        slot_state
            .stage_active_slots
            .get(stage_index)
            .and_then(|queue| queue.back().copied())
            .or_else(|| {
                slot_state
                    .stage_window_stats
                    .get(stage_index)
                    .and_then(|stats| stats.last_completed_slot.or(stats.last_seen_slot))
            })
            .or_else(|| {
                slot_state
                    .stage_last_completed_slot
                    .get(stage_index)
                    .copied()
                    .flatten()
            })
            .map(|slot| slot.to_string())
            .unwrap_or_else(|| "-".to_string())
    }

    fn stage_window_metrics(stage_index: usize, stats: &StageWindowStats) -> String {
        let counter = |name| stats.counters.get(name).copied().unwrap_or(0);
        let avg_ms = if stats.completed > 0 {
            Some(stats.duration_total_ms / stats.completed as u128)
        } else {
            None
        };

        let mut parts = Vec::new();
        match stage_index {
            0 => parts.push(format!("+{}", stats.entered)),
            1 => {
                parts.push(format!("+{}", stats.entered));
                if let Some(avg_ms) = avg_ms {
                    parts.push(format!("avg={}ms", avg_ms));
                }
                if stats.duration_max_ms > 0 {
                    parts.push(format!("max={}ms", stats.duration_max_ms));
                }
            }
            2 => {
                parts.push(format!("+{}", stats.entered));
                if let Some(avg_ms) = avg_ms {
                    parts.push(format!("avg={}ms", avg_ms));
                }
            }
            3 => parts.push(format!("+{}", counter("count"))),
            4 => parts.push(format!("+{}", counter("count"))),
            5 => {
                parts.push(format!("+{}", counter("metadata")));
                parts.push(format!("metadata={}", counter("metadata")));
                parts.push(format!("extracted={}", counter("extracted")));
                parts.push(format!("nested={}", counter("nested")));
            }
            6 => {
                parts.push(format!("+{}", counter("checked")));
                parts.push(format!("pass={}", counter("passed")));
                parts.push(format!("drop={}", counter("dropped")));
            }
            7 => {
                parts.push(format!("+{}", counter("success")));
                parts.push(format!("hit={}", counter("success")));
                parts.push(format!("miss={}", counter("miss")));
            }
            8 => {
                parts.push(format!("+{}", counter("success")));
                parts.push(format!("ok={}", counter("success")));
                parts.push(format!("err={}", counter("error")));
            }
            9 => parts.push(format!("+{}", counter("count"))),
            10 => {
                parts.push(format!("+{}", counter("count")));
                if let Some(avg_ms) = avg_ms {
                    parts.push(format!("avg={}ms", avg_ms));
                }
                if stats.duration_max_ms > 0 {
                    parts.push(format!("max={}ms", stats.duration_max_ms));
                }
            }
            _ => parts.push(format!("+{}", stats.entered)),
        }

        if stats.completed > 0 && stage_index == 0 {
            parts.push(format!("done={}", stats.completed));
        }

        parts.join(" ")
    }

    fn enter_stage_slot(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
    ) -> Option<String> {
        let mut became_visible = false;
        if let Some(stage_activity) = slot_state.stage_slot_activity.get_mut(stage_index) {
            let entry = stage_activity.entry(slot).or_insert(0);
            became_visible = *entry == 0;
            *entry = entry.saturating_add(1);
        }
        if let Some(stage_stats) = slot_state.stage_window_stats.get_mut(stage_index) {
            stage_stats.last_seen_slot = Some(slot);
            if became_visible {
                stage_stats.entered = stage_stats.entered.saturating_add(1);
            }
            Self::record_window_slot(stage_stats, slot);
        }
        if let Some(stage_touched) = slot_state.stage_slot_last_touched.get_mut(stage_index) {
            stage_touched.insert(slot, Instant::now());
        }
        if let Some(queue) = slot_state.stage_active_slots.get_mut(stage_index) {
            queue.retain(|queued_slot| *queued_slot != slot);
            queue.push_back(slot);
        }

        if !became_visible {
            return None;
        }

        Self::ensure_stage_aggregate(slot_state, stage_index, slot);
        None
    }

    fn touch_stage_slot(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
    ) -> Option<String> {
        let is_active = slot_state
            .stage_slot_activity
            .get(stage_index)
            .and_then(|stage_activity| stage_activity.get(&slot))
            .copied()
            .unwrap_or(0)
            > 0;

        if !is_active {
            return Self::enter_stage_slot(slot_state, stage_index, slot);
        }

        if let Some(stage_touched) = slot_state.stage_slot_last_touched.get_mut(stage_index) {
            stage_touched.insert(slot, Instant::now());
        }
        if let Some(queue) = slot_state.stage_active_slots.get_mut(stage_index) {
            queue.retain(|queued_slot| *queued_slot != slot);
            queue.push_back(slot);
        }
        if let Some(stage_stats) = slot_state.stage_window_stats.get_mut(stage_index) {
            stage_stats.last_seen_slot = Some(slot);
            Self::record_window_slot(stage_stats, slot);
        }

        None
    }

    fn exit_stage_slot(
        slot_state: &mut SlotStageState,
        stage_index: usize,
        slot: u64,
    ) -> Option<String> {
        let remaining =
            if let Some(stage_activity) = slot_state.stage_slot_activity.get_mut(stage_index) {
                match stage_activity.get_mut(&slot) {
                    Some(activity) if *activity > 1 => {
                        *activity -= 1;
                        Some(*activity)
                    }
                    Some(_) => {
                        stage_activity.remove(&slot);
                        None
                    }
                    None => None,
                }
            } else {
                None
            };

        if remaining.is_none() {
            let was_active = slot_state.stage_slot_last_touched[stage_index].contains_key(&slot);
            if !was_active {
                return None;
            }
            if let Some(queue) = slot_state.stage_active_slots.get_mut(stage_index) {
                queue.retain(|queued_slot| *queued_slot != slot);
            }
            if let Some(stage_touched) = slot_state.stage_slot_last_touched.get_mut(stage_index) {
                stage_touched.remove(&slot);
            }
            if let Some(aggregate) = slot_state.stage_slot_aggregates[stage_index].remove(&slot) {
                Self::record_stage_completion(slot_state, stage_index, slot, aggregate);
            }
            return None;
        } else if let Some(stage_touched) = slot_state.stage_slot_last_touched.get_mut(stage_index)
        {
            stage_touched.insert(slot, Instant::now());
        }

        None
    }

    fn clear_stage_slot(slot_state: &mut SlotStageState, stage_index: usize, slot: u64) {
        if let Some(stage_activity) = slot_state.stage_slot_activity.get_mut(stage_index) {
            stage_activity.remove(&slot);
        }
        if let Some(queue) = slot_state.stage_active_slots.get_mut(stage_index) {
            queue.retain(|queued_slot| *queued_slot != slot);
        }
        if let Some(stage_touched) = slot_state.stage_slot_last_touched.get_mut(stage_index) {
            stage_touched.remove(&slot);
        }
        if let Some(aggregate) = slot_state.stage_slot_aggregates[stage_index].remove(&slot) {
            Self::record_stage_completion(slot_state, stage_index, slot, aggregate);
        }
    }

    fn slot_has_active_downstream_work(slot_state: &SlotStageState, slot: u64) -> bool {
        (4..SLOT_STAGE_COUNT).any(|stage_index| {
            slot_state.stage_slot_activity[stage_index]
                .get(&slot)
                .copied()
                .unwrap_or(0)
                > 0
        })
    }

    fn finalize_slot_if_drained(slot_state: &mut SlotStageState, slot: u64) -> Vec<String> {
        if !slot_state.sealed_slots.contains(&slot) || Self::slot_has_active_downstream_work(slot_state, slot) {
            return Vec::new();
        }

        let mut lines = Vec::new();
        for stage_index in 4..SLOT_STAGE_COUNT {
            if let Some(aggregate) = slot_state.stage_slot_aggregates[stage_index].remove(&slot) {
                Self::record_stage_completion(slot_state, stage_index, slot, aggregate);
                lines.push(String::new());
            }
        }
        slot_state.sealed_slots.remove(&slot);
        lines.into_iter().filter(|line| !line.is_empty()).collect()
    }

    fn apply_stage_event(slot_state: &mut SlotStageState, name: &str, slot: u64) -> Vec<String> {
        let mut lines = Vec::new();
        match name {
            "stage.schedule_slot.slot" => {
                if let Some(line) = Self::enter_stage_slot(slot_state, 0, slot) {
                    lines.push(line);
                }
            }
            "stage.fetch_block_get_block.slot" => {
                if let Some(line) = Self::exit_stage_slot(slot_state, 0, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 1, slot) {
                    lines.push(line);
                }
            }
            "stage.fetch_block_get_block.exit.slot"
            | "stage.retry_block_fetch.slot"
            | "stage.skip_slot_skippable_rpc_error.slot" => {
                if let Some(line) = Self::exit_stage_slot(slot_state, 1, slot) {
                    lines.push(line);
                }
            }
            "stage.extract_transactions_from_block.slot" => {
                if let Some(line) = Self::enter_stage_slot(slot_state, 2, slot) {
                    lines.push(line);
                }
            }
            "stage.extract_transactions_from_block.exit.slot" => {
                Self::clear_stage_slot(slot_state, 2, slot);
                Self::clear_stage_slot(slot_state, 3, slot);
                slot_state.sealed_slots.insert(slot);
                lines.extend(Self::finalize_slot_if_drained(slot_state, slot));
            }
            "stage.decode_transaction.slot"
            | "stage.transform_transaction_meta.slot"
            | "stage.build_transaction_update.slot"
            | "stage.skip_transaction_decode_or_meta_error.slot"
            | "stage.skip_transaction_failed_status.slot" => {
                if let Some(line) = Self::touch_stage_slot(slot_state, 3, slot) {
                    lines.push(line);
                }
            }
            "stage.enqueue_pipeline_update.slot" => {
                if let Some(line) = Self::touch_stage_slot(slot_state, 3, slot) {
                    lines.push(line);
                }
                Self::bump_window_counter(slot_state, 3, slot, "count", 1);
                Self::bump_stage_counter(slot_state, 4, slot, "count");
                if let Some(line) = Self::enter_stage_slot(slot_state, 4, slot) {
                    lines.push(line);
                }
            }
            "stage.enqueue_pipeline_update.exit.slot"
            | "stage.drop_or_delay_on_channel_backpressure.slot" => {
                if let Some(line) = Self::exit_stage_slot(slot_state, 4, slot) {
                    lines.push(line);
                }
                lines.extend(Self::finalize_slot_if_drained(slot_state, slot));
            }
            "stage.dequeue_pipeline_update.slot" => {
                if let Some(line) = Self::exit_stage_slot(slot_state, 4, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 5, slot) {
                    lines.push(line);
                }
            }
            "stage.build_core_transaction_metadata.slot" => {
                Self::bump_stage_counter(slot_state, 5, slot, "metadata");
                if let Some(line) = Self::touch_stage_slot(slot_state, 5, slot) {
                    lines.push(line);
                }
            }
            "stage.extract_instructions_with_metadata.slot" | "stage.nest_instructions.slot" => {
                let counter = if name == "stage.extract_instructions_with_metadata.slot" {
                    "extracted"
                } else {
                    "nested"
                };
                Self::bump_stage_counter(slot_state, 5, slot, counter);
                if let Some(line) = Self::touch_stage_slot(slot_state, 5, slot) {
                    lines.push(line);
                }
            }
            "stage.apply_instruction_filters.slot" => {
                Self::bump_stage_counter(slot_state, 6, slot, "checked");
                if let Some(line) = Self::exit_stage_slot(slot_state, 5, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 6, slot) {
                    lines.push(line);
                }
            }
            "stage.apply_instruction_filters.exit.slot" => {
                Self::bump_stage_counter(slot_state, 6, slot, "dropped");
                if let Some(line) = Self::exit_stage_slot(slot_state, 6, slot) {
                    lines.push(line);
                }
                lines.extend(Self::finalize_slot_if_drained(slot_state, slot));
            }
            "stage.decode_instruction_jupiter_swap_decoder.slot" => {
                Self::bump_stage_counter(slot_state, 6, slot, "passed");
                if let Some(line) = Self::exit_stage_slot(slot_state, 6, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 7, slot) {
                    lines.push(line);
                }
            }
            "stage.decode_instruction_jupiter_swap_decoder.exit.slot" => {
                Self::bump_stage_counter(slot_state, 7, slot, "miss");
                if let Some(line) = Self::exit_stage_slot(slot_state, 7, slot) {
                    lines.push(line);
                }
                lines.extend(Self::finalize_slot_if_drained(slot_state, slot));
            }
            "stage.process_instruction_postgres_instruction_processor.slot" => {
                Self::bump_stage_counter(slot_state, 7, slot, "success");
                if let Some(line) = Self::exit_stage_slot(slot_state, 7, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 8, slot) {
                    lines.push(line);
                }
            }
            "stage.build_instruction_row_wrapper.slot" => {
                Self::bump_stage_counter(slot_state, 8, slot, "success");
                Self::bump_stage_counter(slot_state, 9, slot, "count");
                if let Some(line) = Self::exit_stage_slot(slot_state, 8, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 9, slot) {
                    lines.push(line);
                }
            }
            "stage.upsert_instruction_row_postgres.slot" => {
                Self::bump_stage_counter(slot_state, 10, slot, "count");
                if let Some(line) = Self::exit_stage_slot(slot_state, 9, slot) {
                    lines.push(line);
                }
                if let Some(line) = Self::enter_stage_slot(slot_state, 10, slot) {
                    lines.push(line);
                }
            }
            "stage.upsert_instruction_row_postgres.exit.slot" => {
                if let Some(line) = Self::exit_stage_slot(slot_state, 10, slot) {
                    lines.push(line);
                }
                lines.extend(Self::finalize_slot_if_drained(slot_state, slot));
            }
            _ => {}
        }

        lines
    }

    fn cleanup_stale_slots(slot_state: &mut SlotStageState) {
        let now = Instant::now();
        for stage_index in 0..SLOT_STAGE_COUNT {
            let stale_slots: Vec<u64> = slot_state
                .stage_slot_last_touched
                .get(stage_index)
                .into_iter()
                .flat_map(|slots| slots.iter())
                .filter_map(|(slot, seen_at)| {
                    if now.duration_since(*seen_at) > SLOT_STAGE_FALLBACK_TTL {
                        Some(*slot)
                    } else {
                        None
                    }
                })
                .collect();
            for slot in stale_slots {
                Self::clear_stage_slot(slot_state, stage_index, slot);
            }
        }
    }

    fn report_snapshot(
        slot_state: &SlotStageState,
        counter_deltas: &HashMap<String, u64>,
        interval_ms: u64,
    ) -> ReportSnapshot {
        let window = Duration::from_millis(interval_ms);
        let queued = slot_state
            .stage_slot_activity
            .iter()
            .map(HashMap::len)
            .sum::<usize>();
        let processed = counter_deltas.get("updates_processed").copied().unwrap_or(0);
        let failed = counter_deltas.get("updates_failed").copied().unwrap_or(0);
        let label_width = Self::stage_label_width();

        let rows = (0..SLOT_STAGE_COUNT)
            .map(|stage_index| {
                let stage_name = Self::stage_display_name_by_index(stage_index);
                let queue = slot_state
                    .stage_active_slots
                    .get(stage_index)
                    .cloned()
                    .unwrap_or_default();
                let active_count = queue.len();
                let display_slot = Self::stage_display_slot(slot_state, stage_index);
                let inflight_slots = Self::format_stage_slots(&queue);
                let window_slots = Self::stage_window_slots(slot_state, stage_index);
                let metrics = slot_state
                    .stage_window_stats
                    .get(stage_index)
                    .map(|stats| Self::stage_window_metrics(stage_index, stats))
                    .unwrap_or_else(|| "+0".to_string());

                format!(
                    "{stage_name:<label_width$} slot={display_slot} active={active_count} inflight={inflight_slots:<32} window={window_slots:<32} {metrics}",
                )
            })
            .collect();

        ReportSnapshot {
            header: format!(
                "pipeline_window t={} window={}s queued={} processed={} failed={}",
                Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                window.as_secs(),
                queued,
                processed,
                failed
            ),
            rows,
        }
    }

    async fn run_housekeeping(&self) {
        let mut slot_state = Self::lock_unpoisoned(&self.slot_state);
        Self::cleanup_stale_slots(&mut slot_state);
    }

    async fn snapshot_report(&self, interval_ms: u64) -> Option<ReportSnapshot> {
        let counter_deltas = {
            let counters = Self::lock_unpoisoned(&self.counters);
            let mut snapshots = Self::lock_unpoisoned(&self.report_counter_snapshot);
            let mut deltas = HashMap::new();
            for (name, value) in counters.iter() {
                let previous = snapshots.get(name).copied().unwrap_or(0);
                deltas.insert(name.clone(), value.saturating_sub(previous));
                snapshots.insert(name.clone(), *value);
            }
            deltas
        };

        let snapshot = {
            let mut slot_state = Self::lock_unpoisoned(&self.slot_state);
            let snapshot = Self::report_snapshot(&slot_state, &counter_deltas, interval_ms);
            for stats in &mut slot_state.stage_window_stats {
                *stats = StageWindowStats::default();
            }
            snapshot
        };

        Some(snapshot)
    }
}

#[async_trait]
impl Metrics for StageSlotMetrics {
    async fn initialize(&self) -> CarbonResult<()> {
        Ok(())
    }

    async fn flush(&self) -> CarbonResult<()> {
        let mut slot_state = StageSlotMetrics::lock_unpoisoned(&self.slot_state);
        Self::cleanup_stale_slots(&mut slot_state);

        Ok(())
    }

    async fn shutdown(&self) -> CarbonResult<()> {
        Ok(())
    }

    async fn update_gauge(&self, name: &str, value: f64) -> CarbonResult<()> {
        self.run_started.store(true, Ordering::Relaxed);
        let slot = value as u64;

        if name.starts_with("stage.") && name.ends_with(".slot") {
            let mut slot_state = StageSlotMetrics::lock_unpoisoned(&self.slot_state);
            let _ = Self::apply_stage_event(&mut slot_state, name, slot);
        }
        let mut gauges = StageSlotMetrics::lock_unpoisoned(&self.gauges);
        gauges.insert(name.to_string(), value);
        Ok(())
    }

    async fn increment_counter(&self, name: &str, value: u64) -> CarbonResult<()> {
        let mut counters = StageSlotMetrics::lock_unpoisoned(&self.counters);
        let entry = counters.entry(name.to_string()).or_insert(0);
        *entry = entry.saturating_add(value);
        Ok(())
    }

    async fn record_histogram(&self, name: &str, value: f64) -> CarbonResult<()> {
        let mut histograms_last = StageSlotMetrics::lock_unpoisoned(&self.histograms_last);
        histograms_last.insert(name.to_string(), value);
        Ok(())
    }
}

fn init_logging() -> CarbonResult<()> {
    let log_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("logs");
    fs::create_dir_all(&log_dir).map_err(|err| {
        CarbonError::Custom(format!("Failed to create log directory {log_dir:?}: {err}"))
    })?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| CarbonError::Custom(format!("Failed to read system time: {err}")))?
        .as_secs();
    let log_file_path = log_dir.join(format!("run-{timestamp}.log"));
    let log_file = File::create(&log_file_path).map_err(|err| {
        CarbonError::Custom(format!(
            "Failed to create log file {log_file_path:?}: {err}"
        ))
    })?;

    let log_level = parse_log_level("LOG_LEVEL", LevelFilter::Info);
    let terminal_log_level = parse_log_level("TERM_LOG_LEVEL", LevelFilter::Warn);
    let mut config_builder = ConfigBuilder::new();

    // Keep high-volume transport/query internals visible only at TRACE.
    if log_level != LevelFilter::Trace {
        config_builder
            .add_filter_ignore_str("reqwest::connect")
            .add_filter_ignore_str("sqlx::query")
            .add_filter_ignore_str("hyper_util::client::legacy::connect::http")
            .add_filter_ignore_str("hyper_util::client::legacy::pool");
    }

    let config = config_builder.build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = Vec::new();
    if terminal_log_level != LevelFilter::Off {
        loggers.push(TermLogger::new(
            terminal_log_level,
            config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }
    loggers.push(WriteLogger::new(log_level, config, log_file));

    CombinedLogger::init(loggers)
        .map_err(|err| CarbonError::Custom(format!("Failed to initialize logger: {err}")))?;

    log::info!(
        "File logging enabled at {} with file level {:?} and terminal level {:?}",
        log_file_path.display(),
        log_level,
        terminal_log_level
    );
    Ok(())
}

const DATASOURCE_ENV_KEY: &str = "DATASOURCE";
const TRANSACTION_DATASOURCE_VALUE: &str = "rpc_transaction_crawler";
const BLOCK_DATASOURCE_VALUE: &str = "rpc_block_crawler";
const DEFAULT_RATE_LIMIT: u32 = 10;
const TAIL_DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 10;
const BACKFILL_TAIL_DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 6;
const RANGE_DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 10;

enum DatasourceMode {
    TransactionCrawler,
    BlockCrawler,
}

enum DatasourceSelection {
    TransactionCrawler(Box<RpcTransactionCrawler>),
    BlockCrawler(RpcBlockCrawler),
}

async fn configure_datasource(rpc_url: String) -> CarbonResult<DatasourceSelection> {
    match datasource_mode_from_env()? {
        DatasourceMode::TransactionCrawler => {
            let rate_limit = read_optional_env_var::<u32>("RATE_LIMIT")?
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_RATE_LIMIT);
            log::info!(
                "Using RpcTransactionCrawler datasource (rate limit: {rate_limit} requests/sec)"
            );
            let filters = Filters::new(None, None, None);
            let connection_config = ConnectionConfig::default().with_rate_limit(rate_limit);
            Ok(DatasourceSelection::TransactionCrawler(Box::new(
                RpcTransactionCrawler::new(
                    rpc_url,
                    JUPITER_SWAP_PROGRAM_ID,
                    connection_config,
                    filters,
                    None,
                ),
            )))
        }
        DatasourceMode::BlockCrawler => {
            let env_start_slot = read_optional_env_var::<u64>("BLOCK_CRAWLER_START_SLOT")?;
            let env_end_slot = read_optional_env_var::<u64>("BLOCK_CRAWLER_END_SLOT")?;
            if env_start_slot.is_none() && env_end_slot.is_some() {
                return Err(CarbonError::Custom(
                    "BLOCK_CRAWLER_END_SLOT requires BLOCK_CRAWLER_START_SLOT".to_string(),
                ));
            }

            let (mode_label, start_slot, end_slot) = match (env_start_slot, env_end_slot) {
                (None, None) => ("tail", None, None),
                (Some(start_slot), None) => ("backfill_tail", Some(start_slot), None),
                (Some(start_slot), Some(end_slot)) => ("range", Some(start_slot), Some(end_slot)),
                (None, Some(_)) => unreachable!("validated above"),
            };

            let ordered_tail =
                read_optional_env_var::<bool>("BLOCK_CRAWLER_ORDERED_TAIL")?.unwrap_or(false);
            let env_max_concurrent_requests =
                read_optional_env_var::<usize>("BLOCK_CRAWLER_MAX_CONCURRENT_REQUESTS")?;
            let rate_limit = read_optional_env_var::<u32>("RATE_LIMIT")?
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_RATE_LIMIT);
            let request_throttle = Some(Duration::from_secs_f64(1.0 / rate_limit as f64));
            let default_max_concurrent_requests = match mode_label {
                "tail" => TAIL_DEFAULT_MAX_CONCURRENT_REQUESTS,
                "backfill_tail" => BACKFILL_TAIL_DEFAULT_MAX_CONCURRENT_REQUESTS,
                _ => RANGE_DEFAULT_MAX_CONCURRENT_REQUESTS,
            };
            let max_concurrent_requests = if ordered_tail && mode_label == "tail" {
                1
            } else {
                env_max_concurrent_requests.unwrap_or(default_max_concurrent_requests)
            };

            let block_config = RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Binary),
                max_supported_transaction_version: Some(0),
                ..RpcBlockConfig::default()
            };
            let latest_slot = if mode_label == "tail" {
                let rpc_client = RpcClient::new(rpc_url.clone());
                Some(rpc_client.get_slot().await.map_err(|err| {
                    CarbonError::Custom(format!(
                        "Failed to fetch the most recent slot for RpcBlockCrawler tail startup: {err}"
                    ))
                })?)
            } else {
                None
            };

            log::info!(
                "Using RpcBlockCrawler datasource (mode: {mode_label}, latest_slot: {:?}, start_slot: {:?}, end_slot: {:?}, interval: 100ms, max_concurrent_requests: {}, ordered_tail: {}, rate_limit: {} req/s)",
                latest_slot,
                start_slot,
                end_slot,
                max_concurrent_requests,
                ordered_tail,
                rate_limit,
            );

            let mut crawler = RpcBlockCrawler::new(
                rpc_url,
                start_slot,
                end_slot,
                None,
                block_config,
                Some(max_concurrent_requests),
                None,
            );
            crawler.mode_label = mode_label.to_string();
            crawler.request_throttle = request_throttle;

            Ok(DatasourceSelection::BlockCrawler(crawler))
        }
    }
}

fn datasource_mode_from_env() -> CarbonResult<DatasourceMode> {
    let raw_value = match env::var(DATASOURCE_ENV_KEY) {
        Ok(value) => value,
        Err(VarError::NotPresent) => String::new(),
        Err(err) => {
            return Err(CarbonError::Custom(format!(
                "Failed to read {DATASOURCE_ENV_KEY}: {err}"
            )))
        }
    };

    let normalized = {
        let trimmed = raw_value.trim();
        if trimmed.is_empty() {
            TRANSACTION_DATASOURCE_VALUE.to_string()
        } else {
            trimmed.to_ascii_lowercase()
        }
    };

    match normalized.as_str() {
        TRANSACTION_DATASOURCE_VALUE
        | "transaction"
        | "transaction_crawler"
        | "rpc_transaction" => Ok(DatasourceMode::TransactionCrawler),
        BLOCK_DATASOURCE_VALUE | "block" | "rpc_block" | "block_crawler" => {
            Ok(DatasourceMode::BlockCrawler)
        }
        other => Err(CarbonError::Custom(format!(
            "Unsupported {DATASOURCE_ENV_KEY} value '{other}'. Expected '{TRANSACTION_DATASOURCE_VALUE}' or '{BLOCK_DATASOURCE_VALUE}'."
        ))),
    }
}

fn read_optional_env_var<T>(key: &str) -> CarbonResult<Option<T>>
where
    T: std::str::FromStr,
    T::Err: Display,
{
    match env::var(key) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed
                    .parse::<T>()
                    .map(Some)
                    .map_err(|err| CarbonError::Custom(format!("Failed to parse {key}: {err}")))
            }
        }
        Err(VarError::NotPresent) => Ok(None),
        Err(err) => Err(CarbonError::Custom(format!("Failed to read {key}: {err}"))),
    }
}

fn parse_log_level(key: &str, default: LevelFilter) -> LevelFilter {
    let raw = env::var(key).unwrap_or_else(|_| match key {
        "TERM_LOG_LEVEL" => "Warn".to_string(),
        _ => String::new(),
    });

    match raw.trim().to_ascii_lowercase().as_str() {
        "" => default,
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

fn ensure_debug_stage_metrics_enabled() {
    const KEY: &str = "CARBON_DEBUG_STAGE_METRICS";

    if env::var_os(KEY).is_none() {
        // The periodic stage log depends on these per-stage counters for
        // meaningful downstream progression instead of raw transport noise.
        env::set_var(KEY, "1");
        log::info!("{KEY} not set; defaulting to enabled for stage-progress logging");
    }
}

fn ensure_pipeline_update_timeout_enabled() {
    const KEY: &str = "CARBON_PIPELINE_UPDATE_TIMEOUT_MS";
    const DEFAULT_TIMEOUT_MS: &str = "60000";

    if env::var_os(KEY).is_none() {
        env::set_var(KEY, DEFAULT_TIMEOUT_MS);
        log::info!("{KEY} not set; defaulting to {DEFAULT_TIMEOUT_MS} ms for this example");
    }
}

fn ensure_instruction_pipe_timeout_enabled() {
    const KEY: &str = "CARBON_INSTRUCTION_PIPE_TIMEOUT_MS";
    const DEFAULT_TIMEOUT_MS: &str = "2000";

    if env::var_os(KEY).is_none() {
        env::set_var(KEY, DEFAULT_TIMEOUT_MS);
        log::info!("{KEY} not set; defaulting to {DEFAULT_TIMEOUT_MS} ms for this example");
    }
}

fn postgres_statement_timeout() -> CarbonResult<Duration> {
    Ok(Duration::from_millis(
        read_optional_env_var::<u64>("POSTGRES_STATEMENT_TIMEOUT_MS")?
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_POSTGRES_STATEMENT_TIMEOUT_MS),
    ))
}

fn postgres_lock_timeout() -> CarbonResult<Duration> {
    Ok(Duration::from_millis(
        read_optional_env_var::<u64>("POSTGRES_LOCK_TIMEOUT_MS")?
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_POSTGRES_LOCK_TIMEOUT_MS),
    ))
}

fn postgres_acquire_timeout() -> CarbonResult<Duration> {
    Ok(Duration::from_millis(
        read_optional_env_var::<u64>("POSTGRES_ACQUIRE_TIMEOUT_MS")?
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_POSTGRES_ACQUIRE_TIMEOUT_MS),
    ))
}

fn stage_log_interval() -> CarbonResult<Duration> {
    Ok(Duration::from_millis(
        read_optional_env_var::<u64>("STAGE_LOG_INTERVAL_MS")?
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_STAGE_LOG_INTERVAL_MS),
    ))
}

fn stage_housekeeping_interval() -> CarbonResult<Duration> {
    Ok(Duration::from_millis(
        read_optional_env_var::<u64>("STAGE_HOUSEKEEPING_INTERVAL_MS")?
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_STAGE_HOUSEKEEPING_INTERVAL_MS),
    ))
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logging()?;
    ensure_debug_stage_metrics_enabled();
    ensure_pipeline_update_timeout_enabled();
    ensure_instruction_pipe_timeout_enabled();

    // Database connection and migrations
    let db_url = env::var("DATABASE_URL")
        .map_err(|err| CarbonError::Custom(format!("DATABASE_URL must be set ({err})")))?;
    let postgres_statement_timeout = postgres_statement_timeout()?;
    let postgres_lock_timeout = postgres_lock_timeout()?;
    let postgres_acquire_timeout = postgres_acquire_timeout()?;

    log::info!(
        "Using Postgres timeouts: statement={}ms lock={}ms acquire={}ms",
        postgres_statement_timeout.as_millis(),
        postgres_lock_timeout.as_millis(),
        postgres_acquire_timeout.as_millis(),
    );

    let pool = PgPoolOptions::new()
        .acquire_timeout(postgres_acquire_timeout)
        .after_connect({
            let statement_timeout_ms = postgres_statement_timeout.as_millis();
            let lock_timeout_ms = postgres_lock_timeout.as_millis();
            move |conn, _meta| {
                Box::pin(async move {
                    let statement_timeout_sql =
                        format!("SET statement_timeout = {statement_timeout_ms}");
                    sqlx::query(&statement_timeout_sql)
                        .execute(&mut *conn)
                        .await?;
                    let lock_timeout_sql = format!("SET lock_timeout = {lock_timeout_ms}");
                    sqlx::query(&lock_timeout_sql)
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            }
        })
        .connect(&db_url)
        .await
        .map_err(|err| CarbonError::Custom(format!("Failed to connect to Postgres: {err}")))?;

    let mut migrator = Migrator::default();
    migrator
        .add_migration(Box::new(JupiterSwapInstructionsMigration))
        .map_err(|err| {
            CarbonError::Custom(format!(
                "Failed to add Jupiter instructions migration: {err}"
            ))
        })?;

    let mut conn = pool.acquire().await.map_err(|err| {
        CarbonError::Custom(format!("Failed to acquire Postgres connection: {err}"))
    })?;
    migrator
        .run(&mut *conn, &Plan::apply_all())
        .await
        .map_err(|err| CarbonError::Custom(format!("Failed to run migrations: {err}")))?;

    let rpc_url = env::var("RPC_URL")
        .map_err(|err| CarbonError::Custom(format!("RPC_URL must be set ({err})")))?;
    let datasource = configure_datasource(rpc_url).await?;
    let stage_metrics = Arc::new(StageSlotMetrics::new());
    let stage_log_interval = stage_log_interval()?;
    let stage_housekeeping_interval = stage_housekeeping_interval()?;

    {
        let stage_metrics = Arc::clone(&stage_metrics);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(stage_housekeeping_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            interval.tick().await;
            loop {
                interval.tick().await;
                stage_metrics.run_housekeeping().await;
            }
        });
    }

    {
        let stage_metrics = Arc::clone(&stage_metrics);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(stage_log_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            interval.tick().await;
            loop {
                interval.tick().await;
                if let Some(snapshot) = stage_metrics
                    .snapshot_report(stage_log_interval.as_millis() as u64)
                    .await
                {
                    log::info!("{}", snapshot.header);
                    for row in snapshot.rows {
                        log::info!("{row}");
                    }
                }
            }
        });
    }

    let pipeline_builder = match datasource {
        DatasourceSelection::TransactionCrawler(datasource) => {
            carbon_core::pipeline::Pipeline::builder().datasource(*datasource)
        }
        DatasourceSelection::BlockCrawler(datasource) => {
            carbon_core::pipeline::Pipeline::builder().datasource(datasource)
        }
    };

    pipeline_builder
        .metrics(stage_metrics)
        .metrics_flush_interval(1)
        .instruction(
            JupiterSwapDecoder,
            PostgresInstructionProcessor::<
                JupiterSwapInstruction,
                JupiterSwapInstructionWithMetadata,
            >::new(pool.clone()),
        )
        .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    Ok(())
}
