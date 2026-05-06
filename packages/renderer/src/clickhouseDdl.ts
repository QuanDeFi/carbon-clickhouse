export type ClickHouseDdlMode = 'merge-tree' | 'replicated-merge-tree' | 'distributed';
export type ClickHouseDistributedLocalDdlMode = Exclude<ClickHouseDdlMode, 'distributed'>;
export type ClickHouseTableKind = 'account' | 'instruction' | 'event';

export type ClickHouseFieldOverride<T> =
    | T
    | Partial<Record<ClickHouseTableKind, T>>;

export type ClickHouseEngineSettings =
    | string
    | string[]
    | Record<string, string | number | boolean>;

export type ClickHouseRenderOptions = {
    enabled?: boolean;
    allowJsonFallback?: boolean;
    ddlMode?: ClickHouseDdlMode;
    onCluster?: string;
    partitionBy?: ClickHouseFieldOverride<string>;
    orderBy?: ClickHouseFieldOverride<string | string[]>;
    ttl?: ClickHouseFieldOverride<string>;
    engineSettings?: ClickHouseEngineSettings;
    columnCodecs?: Record<string, string>;
    replicatedTablePath?: string;
    replicaName?: string;
    distributedCluster?: string;
    distributedDatabase?: string;
    distributedLocalTableSuffix?: string;
    distributedLocalDdlMode?: ClickHouseDistributedLocalDdlMode;
    distributedShardingKey?: string;
};

export type ClickHouseOptionInput = boolean | ClickHouseRenderOptions | undefined;

export type ClickHouseDdlContext = {
    mode: ClickHouseDdlMode;
    onClusterClause: string;
    engine: string;
    localEngine: string | null;
    createLocalTable: boolean;
    localTableSuffix: string;
    partitionBy: string;
    orderBy: string;
    ttlClause: string;
    settingsClause: string;
    columnCodecs: Record<string, string>;
};

type NormalizedClickHouseOptions = Required<
    Pick<
        ClickHouseRenderOptions,
        | 'ddlMode'
        | 'distributedLocalTableSuffix'
        | 'distributedLocalDdlMode'
        | 'distributedShardingKey'
        | 'replicatedTablePath'
        | 'replicaName'
    >
> &
    Omit<
        ClickHouseRenderOptions,
        | 'enabled'
        | 'ddlMode'
        | 'distributedLocalTableSuffix'
        | 'distributedLocalDdlMode'
        | 'distributedShardingKey'
        | 'replicatedTablePath'
        | 'replicaName'
    >;

export function isClickHouseEnabled(withClickHouse: ClickHouseOptionInput): boolean {
    if (typeof withClickHouse === 'object' && withClickHouse !== null) {
        return withClickHouse.enabled !== false;
    }
    return withClickHouse === true;
}

export function getClickHouseRenderOptions(withClickHouse: ClickHouseOptionInput): ClickHouseRenderOptions {
    if (typeof withClickHouse === 'object' && withClickHouse !== null) {
        return withClickHouse;
    }
    return {};
}

export function getClickHouseDdlContext(
    withClickHouse: ClickHouseOptionInput,
    kind: ClickHouseTableKind,
): ClickHouseDdlContext {
    const options = normalizeOptions(getClickHouseRenderOptions(withClickHouse));
    const partitionBy = resolveByKind(options.partitionBy, kind) ?? defaultPartitionBy(kind);
    const orderBy = renderOrderBy(resolveByKind(options.orderBy, kind) ?? defaultOrderBy(kind));
    const ttl = resolveByKind(options.ttl, kind);
    const settings = renderSettings(options.engineSettings);
    const onClusterClause = options.onCluster ? ` ON CLUSTER ${options.onCluster}` : '';

    return {
        mode: options.ddlMode,
        onClusterClause,
        engine: renderEngine(options, options.ddlMode, false),
        localEngine:
            options.ddlMode === 'distributed'
                ? renderEngine(options, options.distributedLocalDdlMode, true)
                : null,
        createLocalTable: options.ddlMode === 'distributed',
        localTableSuffix: options.distributedLocalTableSuffix,
        partitionBy,
        orderBy,
        ttlClause: ttl ? ` TTL ${rustFormatFragment(ttl)}` : '',
        settingsClause: settings ? ` SETTINGS ${settings}` : '',
        columnCodecs: options.columnCodecs ?? {},
    };
}

function normalizeOptions(options: ClickHouseRenderOptions): NormalizedClickHouseOptions {
    return {
        ...options,
        ddlMode: options.ddlMode ?? 'merge-tree',
        distributedLocalTableSuffix: options.distributedLocalTableSuffix ?? '_local',
        distributedLocalDdlMode: options.distributedLocalDdlMode ?? 'replicated-merge-tree',
        distributedShardingKey: options.distributedShardingKey ?? 'rand()',
        replicatedTablePath: options.replicatedTablePath ?? '/clickhouse/tables/{shard}/{database}/{table_name}',
        replicaName: options.replicaName ?? '{replica}',
    };
}

function resolveByKind<T>(value: ClickHouseFieldOverride<T> | undefined, kind: ClickHouseTableKind): T | undefined {
    if (typeof value !== 'object' || value === null || Array.isArray(value)) {
        return value as T | undefined;
    }
    return (value as Partial<Record<ClickHouseTableKind, T>>)[kind];
}

function defaultPartitionBy(kind: ClickHouseTableKind): string {
    return kind === 'account' ? 'partition_slot' : 'toYear(partition_time)';
}

function defaultOrderBy(kind: ClickHouseTableKind): string {
    if (kind === 'account') return '(program_id, family_name, account_id, slot)';
    if (kind === 'event') return '(program_id, family_name, event_id, slot)';
    return '(program_id, family_name, instruction_id, slot)';
}

function renderOrderBy(value: string | string[]): string {
    return Array.isArray(value) ? `(${value.join(', ')})` : value;
}

function renderEngine(
    options: NormalizedClickHouseOptions,
    mode: ClickHouseDistributedLocalDdlMode | ClickHouseDdlMode,
    local: boolean,
): string {
    if (mode === 'replicated-merge-tree') {
        return `ReplicatedMergeTree('${rustFormatFragment(options.replicatedTablePath)}', '${rustFormatFragment(
            options.replicaName,
        )}')`;
    }

    if (mode === 'distributed') {
        const cluster = rustFormatFragment(options.distributedCluster ?? options.onCluster ?? 'default');
        const database = rustFormatFragment(options.distributedDatabase ?? 'default');
        const localTable = local
            ? '{table_name}'
            : `{table_name}${rustFormatFragment(options.distributedLocalTableSuffix)}`;
        return `Distributed('${cluster}', '${database}', '${localTable}', ${rustFormatFragment(
            options.distributedShardingKey,
        )})`;
    }

    return 'MergeTree';
}

function renderSettings(settings: ClickHouseEngineSettings | undefined): string {
    if (!settings) return '';
    if (typeof settings === 'string') return rustFormatFragment(settings);
    if (Array.isArray(settings)) return settings.map(rustFormatFragment).join(', ');

    return Object.entries(settings)
        .map(([key, value]) => `${key} = ${renderSettingValue(value)}`)
        .join(', ');
}

function renderSettingValue(value: string | number | boolean): string {
    if (typeof value === 'number') return String(value);
    if (typeof value === 'boolean') return value ? '1' : '0';
    return rustFormatFragment(value);
}

function rustFormatFragment(value: string): string {
    return value
        .replace(/\\/g, '\\\\')
        .replace(/"/g, '\\"')
        .replace(/\{table_name\}/g, '\u0000')
        .replace(/\{/g, '{{')
        .replace(/\}/g, '}}')
        .replace(/\u0000/g, '{table_name}');
}
