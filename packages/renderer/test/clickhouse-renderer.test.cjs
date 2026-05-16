const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');

const { camelCase, kebabCase, pascalCase, snakeCase, titleCase } = require('@codama/nodes');
const nunjucks = require('nunjucks');

const { ClickHouseRowMapper, getClickHouseDdlContext, isClickHouseEnabled } = require('../dist/index.js');

const env = nunjucks.configure('templates', {
    autoescape: false,
    trimBlocks: true,
});

env.addFilter('pascalCase', pascalCase);
env.addFilter('camelCase', camelCase);
env.addFilter('snakeCase', snakeCase);
env.addFilter('kebabCase', kebabCase);
env.addFilter('titleCase', titleCase);
env.addGlobal('RUST_KEYWORDS', []);

const program = { name: 'demoProgram' };
const defaultInstructionDdl = getClickHouseDdlContext(true, 'instruction');
const defaultAccountDdl = getClickHouseDdlContext(true, 'account');
const defaultEventDdl = getClickHouseDdlContext(true, 'event');
const typedField = {
    column: 'amount',
    rowType: 'u64',
    clickHouseColumnType: 'UInt64',
    expr: 'source.amount',
};

function render(template, context) {
    return env.render(template, context);
}

function readFilesRecursively(dir) {
    const out = [];
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
        const entryPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            out.push(...readFilesRecursively(entryPath));
        } else {
            out.push(entryPath);
        }
    }
    return out;
}

function assertNoGeneratedClickHouseJsonFallback(relativeDir) {
    const dir = path.resolve(__dirname, '../../../', relativeDir);
    const files = readFilesRecursively(dir).filter(file => file.endsWith('.rs'));
    assert.ok(files.length > 0, `expected generated ClickHouse files under ${relativeDir}`);

    for (const file of files) {
        const output = fs.readFileSync(file, 'utf8');
        assert.doesNotMatch(output, /\b[a-zA-Z_][a-zA-Z0-9_]*\s+JSON\b/, `${file} contains a JSON column`);
        assert.doesNotMatch(
            output,
            /pub\s+[a-zA-Z_][a-zA-Z0-9_]*:\s*serde_json::Value\b/,
            `${file} contains a serde_json::Value row field`,
        );
    }
}

{
    const output = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [{ name: 'swapEvent' }],
        hasAnchorEvents: true,
    });

    assert.match(output, /pub mod swap_row;/);
    assert.match(output, /pub mod cpi_event_row;/);
    assert.doesNotMatch(output, /pub mod swap_event_event_row;/);
    assert.match(output, /SwapInstructionClickHouseRow/);
    assert.match(output, /DemoProgramCpiEventClickHouseRow/);
    assert.match(output, /ClickHouseRows<DemoProgramClickHouseInstructionRow>/);
    assert.match(output, /DemoProgramInstruction::Swap/);
    assert.match(output, /super::CpiEvent::SwapEvent/);
    assert.match(output, /migration_operations\(swap_row::SwapInstructionClickHouseRow::DEFAULT_TABLE_NAME\)/);
    assert.match(output, /migration_operations\(cpi_event_row::DemoProgramCpiEventClickHouseRow::DEFAULT_TABLE_NAME\)/);
}

{
    const output = render('accountsClickHouseMod.njk', {
        program,
        accountsToExport: [{ name: 'mint' }, { name: 'token' }],
    });

    assert.match(output, /pub mod mint_row;/);
    assert.match(output, /pub mod token_row;/);
    assert.match(output, /MintAccountClickHouseRow::migration_operations/);
    assert.match(output, /TokenAccountClickHouseRow::migration_operations/);
    assert.match(output, /ClickHouseAccountProcessor</);
    assert.match(output, /DemoProgramAccount::Mint/);
}

{
    const output = render('clickhouseRowPage.njk', {
        program,
        entityName: 'mint',
        isAccount: true,
        flatFields: [typedField],
        clickHouseDdl: defaultAccountDdl,
    });

    assert.match(output, /pub struct MintAccountClickHouseRow/);
    assert.match(output, /deterministic_account_id/);
    assert.match(output, /pub amount: u64/);
    assert.match(output, /amount UInt64/);
    assert.match(output, /ALTER TABLE \{table_name\} ADD COLUMN IF NOT EXISTS amount UInt64/);
    assert.match(output, /demo_program_mint_account_landing/);
}

{
    const output = render('cpiEventClickHouseRowPage.njk', {
        program,
        clickHouseDdl: defaultEventDdl,
        clickHouseEventHelperDefinitions: [],
        clickHouseEventUnionPlans: [
            {
                name: 'feeEvent',
                fields: [
                    {
                        column: 'fee_event_amount',
                        rowType: 'Option<u64>',
                        clickHouseColumnType: 'Nullable(UInt64)',
                        expr: 'Some(source.amount)',
                        defaultExpr: 'None',
                    },
                ],
            },
            {
                name: 'swapsEvent',
                fields: [
                    {
                        column: 'swaps_event_swap_events_present',
                        rowType: 'bool',
                        clickHouseColumnType: 'Bool',
                        expr: 'true',
                        defaultExpr: 'false',
                    },
                    {
                        column: 'swaps_event_swap_events',
                        rowType: 'Vec<ClickHouseSwapEvent>',
                        clickHouseColumnType: 'Array(Tuple(input_mint String, input_amount UInt64))',
                        expr: 'source.swap_events.iter().map(ClickHouseSwapEvent::from).collect()',
                        defaultExpr: 'Vec::new()',
                    },
                ],
            },
        ],
        clickHouseEventUnionFields: [
            {
                column: 'fee_event_amount',
                rowType: 'Option<u64>',
                clickHouseColumnType: 'Nullable(UInt64)',
                expr: 'Some(source.amount)',
                defaultExpr: 'None',
            },
            {
                column: 'swaps_event_swap_events_present',
                rowType: 'bool',
                clickHouseColumnType: 'Bool',
                expr: 'true',
                defaultExpr: 'false',
            },
            {
                column: 'swaps_event_swap_events',
                rowType: 'Vec<ClickHouseSwapEvent>',
                clickHouseColumnType: 'Array(Tuple(input_mint String, input_amount UInt64))',
                expr: 'source.swap_events.iter().map(ClickHouseSwapEvent::from).collect()',
                defaultExpr: 'Vec::new()',
            },
        ],
    });

    assert.match(output, /pub struct DemoProgramCpiEventClickHouseRow/);
    assert.match(output, /pub const DEFAULT_TABLE_NAME: &'static str = "demo_program_cpi_event_landing"/);
    assert.match(output, /pub fee_event_amount: Option<u64>/);
    assert.match(output, /fee_event_amount Nullable\(UInt64\)/);
    assert.match(output, /swaps_event_swap_events_present Bool/);
    assert.match(output, /swaps_event_swap_events Array\(Tuple\(input_mint String, input_amount UInt64\)\)/);
    assert.match(output, /pub fn from_fee_event/);
    assert.match(output, /pub fn from_swaps_event/);
    assert.doesNotMatch(output, /data JSON/);
}

{
    assert.equal(isClickHouseEnabled(true), true);
    assert.equal(isClickHouseEnabled({ ddlMode: 'replicated-merge-tree' }), true);
    assert.equal(isClickHouseEnabled({ enabled: false, ddlMode: 'merge-tree' }), false);

    const replicatedDdl = getClickHouseDdlContext(
        {
            ddlMode: 'replicated-merge-tree',
            onCluster: 'prod_cluster',
            partitionBy: { instruction: 'toYYYYMM(partition_time)' },
            orderBy: { instruction: ['program_id', 'slot', 'instruction_id'] },
            ttl: { instruction: 'partition_time + INTERVAL 30 DAY' },
            engineSettings: { index_granularity: 8192 },
            columnCodecs: { amount: 'ZSTD(3)' },
            replicatedTablePath: '/clickhouse/{cluster}/{table_name}',
            replicaName: '{replica}',
        },
        'instruction',
    );
    const replicatedOutput = render('clickhouseRowPage.njk', {
        program,
        entityName: 'swap',
        isAccount: false,
        flatFields: [typedField],
        clickHouseDdl: replicatedDdl,
    });

    assert.match(replicatedOutput, /ON CLUSTER prod_cluster/);
    assert.match(replicatedOutput, /ENGINE = \{engine\}\{merge_tree_clauses\}/);
    assert.match(replicatedOutput, /ReplicatedMergeTree\('\/clickhouse\/\{\{cluster\}\}\/\{table_name\}', '\{\{replica\}\}'\)/);
    assert.match(replicatedOutput, /PARTITION BY toYYYYMM\(partition_time\) ORDER BY \(program_id, slot, instruction_id\)/);
    assert.match(replicatedOutput, /TTL partition_time \+ INTERVAL 30 DAY SETTINGS index_granularity = 8192/);
    assert.match(replicatedOutput, /amount UInt64 CODEC\(ZSTD\(3\)\)/);
    assert.match(replicatedOutput, /ALTER TABLE \{table_name\} ON CLUSTER prod_cluster ADD COLUMN IF NOT EXISTS amount UInt64 CODEC\(ZSTD\(3\)\)/);

    const distributedDdl = getClickHouseDdlContext(
        {
            ddlMode: 'distributed',
            onCluster: 'prod_cluster',
            distributedCluster: 'prod_cluster',
            distributedDatabase: 'analytics',
            distributedShardingKey: 'cityHash64(instruction_id)',
            distributedLocalTableSuffix: '_local',
        },
        'instruction',
    );
    const distributedOutput = render('clickhouseRowPage.njk', {
        program,
        entityName: 'swap',
        isAccount: false,
        flatFields: [typedField],
        clickHouseDdl: distributedDdl,
    });

    assert.match(distributedOutput, /pub fn create_local_table_sql\(table_name: &str\) -> String/);
    assert.match(distributedOutput, /format!\("\{table_name\}_local"\)/);
    assert.match(distributedOutput, /Distributed\('prod_cluster', 'analytics', '\{table_name\}_local', cityHash64\(instruction_id\)\)/);
    assert.match(distributedOutput, /Self::create_table_sql_for\(table_name, &engine, false\)/);
    assert.match(distributedOutput, /Self::create_table_sql_for\(&local_table_name, &engine, true\)/);
}

{
    const cargoTomlGenerator = fs.readFileSync('src/cargoTomlGenerator.ts', 'utf8');

    assert.match(cargoTomlGenerator, /clickhouse = \[/);
    assert.match(cargoTomlGenerator, /"carbon-core\/clickhouse"/);
    assert.match(cargoTomlGenerator, /"serde"/);
    assert.match(cargoTomlGenerator, /"dep:chrono"/);
    assert.match(cargoTomlGenerator, /const chronoDep = getCrateDependencyString\('chrono'/);
    assert.match(cargoTomlGenerator, /dependencies\.push\(chronoDep\)/);
}

{
    const number = format => ({ kind: 'numberTypeNode', format, endian: 'le' });
    const bool = { kind: 'booleanTypeNode', size: number('u8') };
    const field = (name, type) => ({ kind: 'structFieldTypeNode', name, type });
    const struct = fields => ({ kind: 'structTypeNode', fields });
    const tuple = items => ({ kind: 'tupleTypeNode', items });
    const array = item => ({ kind: 'arrayTypeNode', item, count: { kind: 'remainderCountNode' } });
    const option = item => ({ kind: 'optionTypeNode', item, prefix: number('u8') });
    const defined = name => ({ kind: 'definedTypeLinkNode', name });
    const emptyVariant = name => ({ kind: 'enumEmptyVariantTypeNode', name });
    const structVariant = (name, fields) => ({ kind: 'enumStructVariantTypeNode', name, struct: struct(fields) });
    const tupleVariant = (name, items) => ({ kind: 'enumTupleVariantTypeNode', name, tuple: { kind: 'tupleTypeNode', items } });

    const side = {
        kind: 'enumTypeNode',
        variants: [emptyVariant('bid'), emptyVariant('ask')],
        size: number('u8'),
    };
    const remainingAccountsSlice = struct([field('accountsType', number('u8')), field('length', number('u8'))]);
    const remainingAccountsInfo = struct([field('slices', array(defined('remainingAccountsSlice')))]);
    const candidateSwap = {
        kind: 'enumTypeNode',
        variants: [
            structVariant('humidiFi', [field('swapId', number('u64')), field('isBaseToQuote', bool)]),
            structVariant('humidiFiV2', [field('swapId', number('u64')), field('isBaseToQuote', bool)]),
            structVariant('tesseraV', [field('side', defined('side'))]),
            emptyVariant('raydiumV2'),
        ],
        size: number('u8'),
    };
    const swap = {
        kind: 'enumTypeNode',
        variants: [
            emptyVariant('saber'),
            structVariant('humidiFi', [field('swapId', number('u64')), field('isBaseToQuote', bool)]),
            structVariant('scorch', [field('swapId', number('u128'))]),
            structVariant('whirlpoolSwapV2', [
                field('aToB', bool),
                field('remainingAccountsInfo', option(defined('remainingAccountsInfo'))),
            ]),
            structVariant('meteoraDlmmSwapV2', [field('remainingAccountsInfo', defined('remainingAccountsInfo'))]),
            structVariant('dynamicV1', [
                field('candidateSwaps', array(defined('candidateSwap'))),
                field('bestPosition', option(number('u8'))),
            ]),
            tupleVariant('tuplePayload', [number('u64'), bool]),
        ],
        size: number('u8'),
    };
    const routePlanStep = struct([
        field('swap', defined('swap')),
        field('percent', number('u8')),
        field('inputIndex', number('u8')),
        field('outputIndex', number('u8')),
    ]);

    const definedTypes = new Map([
        ['side', { name: 'side', type: side }],
        ['remainingAccountsSlice', { name: 'remainingAccountsSlice', type: remainingAccountsSlice }],
        ['remainingAccountsInfo', { name: 'remainingAccountsInfo', type: remainingAccountsInfo }],
        ['candidateSwap', { name: 'candidateSwap', type: candidateSwap }],
        ['swap', { name: 'swap', type: swap }],
        ['routePlanStep', { name: 'routePlanStep', type: routePlanStep }],
    ]);

    const mapper = new ClickHouseRowMapper({ getDefinedTypesMap: () => definedTypes });
    const plan = mapper.planType(
        struct([
            field('routePlan', array(defined('routePlanStep'))),
            field('tuplePair', tuple([number('u64'), bool])),
            field('tuplePairs', array(tuple([number('u8'), bool]))),
        ]),
        [],
        [],
        new Set(),
    );
    const output = [JSON.stringify(plan.fields), ...plan.helperDefinitions].join('\n');

    assert.match(output, /pub struct ClickHouseSwap/);
    assert.match(output, /pub struct ClickHouseRoutePlanStep/);
    assert.match(output, /pub struct ClickHouseCandidateSwap/);
    assert.match(output, /pub struct ClickHouseSourceTuplePair/);
    assert.match(output, /pub struct ClickHouseUInt128/);
    assert.match(output, /route_plan.*Array\(Tuple/);
    assert.match(output, /tuple_pair.*Tuple\(value_0 UInt64, value_1 Bool\)/);
    assert.match(output, /tuple_pairs.*Array\(Tuple\(value_0 UInt8, value_1 Bool\)\)/);
    assert.match(output, /variant Enum8\('Saber' = 0/);
    assert.match(output, /swap_id: Option<ClickHouseUInt128>/);
    assert.match(output, /swap_id Nullable\(UInt128\)/);
    assert.match(output, /remaining_accounts_info_present Bool/);
    assert.match(output, /candidate_swaps_present Bool/);
    assert.match(output, /candidate_swaps Array\(Tuple\(variant Enum8/);
    assert.match(output, /side Nullable\(Enum8\('Bid' = 0, 'Ask' = 1\)\)/);
    assert.match(output, /value_0 Nullable\(UInt64\)/);
    assert.match(output, /row\.remaining_accounts_info = ClickHouseRemainingAccountsInfo::from\(value\);/);
    assert.match(output, /row\.best_position = best_position\.as_ref\(\)\.map\(\|value\| \*value\);/);
    assert.match(output, /row\.swap_id = Some\(ClickHouseUInt128\(\(\*swap_id\) as u128\)\);/);
    assert.doesNotMatch(output, /payload String/);
    assert.doesNotMatch(output, /clickhouse_enum_json/);
    assert.doesNotMatch(output, /serde_json::Value/);
}

{
    const number = format => ({ kind: 'numberTypeNode', format, endian: 'le' });
    const field = (name, type) => ({ kind: 'structFieldTypeNode', name, type });
    const struct = fields => ({ kind: 'structTypeNode', fields });
    const unsupportedMap = {
        kind: 'mapTypeNode',
        key: { kind: 'stringTypeNode', encoding: 'utf8' },
        value: number('u64'),
    };

    const strictMapper = new ClickHouseRowMapper({ getDefinedTypesMap: () => new Map() });
    assert.throws(
        () => strictMapper.planType(struct([field('rawPayload', unsupportedMap)]), [], [], new Set()),
        /Unsupported ClickHouse type "mapTypeNode" at "source\.raw_payload"/,
    );

    const fallbackMapper = new ClickHouseRowMapper({
        getDefinedTypesMap: () => new Map(),
        allowJsonFallback: true,
    });
    const plan = fallbackMapper.planType(struct([field('rawPayload', unsupportedMap)]), [], [], new Set());
    assert.equal(plan.fields[0].clickHouseColumnType, 'JSON');
    assert.equal(plan.fields[0].rowType, 'serde_json::Value');
    assert.match(plan.fields[0].expr, /serde_json::to_value\(source\.raw_payload\)/);
}

{
    assertNoGeneratedClickHouseJsonFallback('decoders/jupiter-swap-decoder/src/instructions/clickhouse');
    assertNoGeneratedClickHouseJsonFallback('decoders/token-program-decoder/src/accounts/clickhouse');
}

{
    const accountsMod = render('accountsMod.njk', {
        program,
        accountsToExport: [{ name: 'mint' }],
        withPostgres: true,
        withGraphQL: true,
        withClickHouse: false,
        postgresMode: 'typed',
        imports: '',
    });
    const instructionsMod = render('instructionsMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        hasAnchorEvents: false,
        withPostgres: true,
        withGraphQL: true,
        withClickHouse: false,
        postgresMode: 'typed',
        imports: '',
    });

    assert.doesNotMatch(accountsMod, /pub mod clickhouse/);
    assert.doesNotMatch(instructionsMod, /pub mod clickhouse/);
    assert.match(accountsMod, /pub mod postgres/);
    assert.match(instructionsMod, /pub mod postgres/);
    assert.match(accountsMod, /pub mod graphql/);
    assert.match(instructionsMod, /pub mod graphql/);
}

console.log('clickhouse renderer tests passed');
