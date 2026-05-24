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
        hasClickHouseInstructionTypes: true,
        clickHouseDdl: defaultInstructionDdl,
        clickHouseEventDdl: defaultEventDdl,
    });

    assert.match(output, /pub mod types;/);
    assert.match(output, /pub mod swap_row;/);
    assert.match(output, /pub mod swap_event_event_row;/);
    assert.doesNotMatch(output, /pub mod cpi_event_row;/);
    assert.match(output, /pub use self::swap_row::SwapInstructionClickHouseRow;/);
    assert.match(output, /pub use self::swap_event_event_row::SwapEventEventClickHouseRow;/);
    assert.doesNotMatch(output, /pub use self::swap_row::\*/);
    assert.doesNotMatch(output, /pub use self::swap_event_event_row::\*/);
    assert.match(output, /SwapInstructionClickHouseRow/);
    assert.match(output, /SwapEventEventClickHouseRow/);
    assert.match(output, /instruction\s+DemoProgramClickHouseInstructionRow/);
    assert.match(output, /DemoProgramInstructionWithClickHouseMetadata/);
    assert.match(output, /DemoProgramInstruction/);
    assert.match(output, /events CpiEvent/);
    assert.match(output, /carbon_core::clickhouse_row_dispatch!/);
    assert.match(output, /Swap => swap_row::SwapInstructionClickHouseRow/);
    assert.match(output, /SwapEventEvent: SwapEvent => swap_event_event_row::SwapEventEventClickHouseRow/);
    assert.match(output, /fn clickhouse_instruction_table_options\(\) -> ClickHouseTableOptions/);
    assert.match(output, /fn clickhouse_event_table_options\(\) -> ClickHouseTableOptions/);
    assert.match(output, /order_by: r#"\(program_id, family_name, event_id, slot\)"#/);
}

{
    const output = render('accountsClickHouseMod.njk', {
        program,
        accountsToExport: [{ name: 'mint' }, { name: 'token' }],
        hasClickHouseAccountTypes: true,
        clickHouseDdl: defaultAccountDdl,
    });

    assert.match(output, /pub mod types;/);
    assert.match(output, /pub mod mint_row;/);
    assert.match(output, /pub mod token_row;/);
    assert.match(output, /pub use self::mint_row::MintAccountClickHouseRow;/);
    assert.match(output, /pub use self::token_row::TokenAccountClickHouseRow;/);
    assert.doesNotMatch(output, /pub use self::mint_row::\*/);
    assert.doesNotMatch(output, /pub use self::token_row::\*/);
    assert.match(output, /carbon_core::clickhouse_row_dispatch!/);
    assert.match(output, /account\s+DemoProgramClickHouseAccountRow/);
    assert.match(output, /Mint => mint_row::MintAccountClickHouseRow/);
    assert.match(output, /Token => token_row::TokenAccountClickHouseRow/);
    assert.match(output, /fn clickhouse_account_table_options\(\) -> ClickHouseTableOptions/);
    assert.match(output, /ClickHouseAccountProcessor</);
    assert.match(output, /DemoProgramAccount/);
}

{
    const output = render('clickhouseTypesPage.njk', {
        helperDefinitions: ['pub struct ClickHouseSharedHelper { pub value: u64 }'],
    });

    assert.match(output, /use carbon_core::clickhouse::rows::clickhouse_enum_variant;/);
    assert.match(output, /pub struct ClickHouseSharedHelper/);
}

{
    const mapper = new ClickHouseRowMapper({
        getDefinedTypesMap: () =>
            new Map([
                [
                    'routePlanStep',
                    {
                        type: {
                            kind: 'structTypeNode',
                            fields: [
                                {
                                    kind: 'structFieldTypeNode',
                                    name: 'percent',
                                    type: { kind: 'numberTypeNode', format: 'u8', endian: 'le' },
                                },
                            ],
                        },
                    },
                ],
            ]),
    });
    const plan = mapper.planType(
        {
            kind: 'structTypeNode',
            fields: [
                {
                    kind: 'structFieldTypeNode',
                    name: 'routePlan',
                    type: {
                        kind: 'arrayTypeNode',
                        item: { kind: 'definedTypeLinkNode', name: 'routePlanStep' },
                    },
                },
            ],
        },
        [],
        [],
        new Set(),
    );

    assert.equal(plan.fields[0].clickHouseColumnType, 'Array(Tuple(percent UInt8))');
    assert.equal(plan.fields[0].clickHouseColumnTypeExpr, 'ROUTE_PLAN_STEP_ARRAY_DDL');
    assert.ok(plan.helperDefinitions.includes('pub const ROUTE_PLAN_STEP_ARRAY_DDL: &str = r#"Array(Tuple(percent UInt8))"#;'));
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
    assert.match(output, /ClickHouseAccountLandingMetadata::new/);
    assert.match(output, /#\[serde\(flatten\)\]/);
    assert.match(output, /pub amount: u64/);
    assert.match(output, /ClickHouseColumnSpec::new\("amount", r#"UInt64"#\)/);
    assert.match(output, /carbon_core::impl_clickhouse_account_row!\(\s*MintAccountClickHouseRow\s*\);/);
    assert.doesNotMatch(output, /impl_clickhouse_account_row!\(\s*MintAccountClickHouseRow\s*,/);
    assert.match(output, /demo_program_mint_account_landing/);
}

{
    const output = render('clickhouseRowPage.njk', {
        program,
        entityName: 'swap',
        isAccount: false,
        flatFields: [typedField],
        clickHouseDdl: defaultInstructionDdl,
    });

    assert.match(output, /pub struct SwapInstructionClickHouseRow/);
    assert.match(output, /carbon_core::impl_clickhouse_instruction_row!\(\s*SwapInstructionClickHouseRow\s*\);/);
    assert.doesNotMatch(output, /impl_clickhouse_instruction_row!\(\s*SwapInstructionClickHouseRow\s*,/);
}

{
    const output = render('eventInstructionClickHouseRowPage.njk', {
        program,
        event: { name: 'swapsEvent' },
        helperDefinitions: [],
        flatFields: [
            {
                column: 'swap_events_present',
                rowType: 'bool',
                clickHouseColumnType: 'Bool',
                expr: 'true',
            },
            {
                column: 'swap_events',
                rowType: 'Vec<ClickHouseSwapEvent>',
                clickHouseColumnType: 'Array(Tuple(input_mint String, input_amount UInt64))',
                expr: 'source.swap_events.iter().map(ClickHouseSwapEvent::from).collect()',
            },
        ],
        clickHouseDdl: defaultEventDdl,
    });

    assert.match(output, /pub struct SwapsEventEventClickHouseRow/);
    assert.match(output, /pub const DEFAULT_TABLE_NAME: &'static str = "demo_program_swaps_event_landing"/);
    assert.match(output, /ClickHouseEventLandingMetadata::new/);
    assert.match(output, /pub swap_events_present: bool/);
    assert.match(output, /pub swap_events: Vec<ClickHouseSwapEvent>/);
    assert.match(output, /ClickHouseColumnSpec::new\("swap_events_present", r#"Bool"#\)/);
    assert.match(
        output,
        /ClickHouseColumnSpec::new\("swap_events", r#"Array\(Tuple\(input_mint String, input_amount UInt64\)\)"#\)/,
    );
    assert.match(output, /carbon_core::impl_clickhouse_event_row!\(\s*SwapsEventEventClickHouseRow\s*\);/);
    assert.doesNotMatch(output, /impl_clickhouse_event_row!\(\s*SwapsEventEventClickHouseRow\s*,/);
    assert.match(output, /pub fn from_parts/);
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

    const replicatedModOutput = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [],
        hasAnchorEvents: false,
        hasClickHouseInstructionTypes: false,
        clickHouseDdl: replicatedDdl,
        clickHouseEventDdl: getClickHouseDdlContext(true, 'event'),
    });

    assert.match(replicatedModOutput, /ON CLUSTER prod_cluster/);
    assert.match(
        replicatedModOutput,
        /engine: r#"ReplicatedMergeTree\('\/clickhouse\/\{\{cluster\}\}\/\{table_name\}', '\{\{replica\}\}'\)"#\.to_string\(\)/,
    );
    assert.match(replicatedModOutput, /partition_by: r#"toYYYYMM\(partition_time\)"#/);
    assert.match(replicatedModOutput, /order_by: r#"\(program_id, slot, instruction_id\)"#/);
    assert.match(replicatedModOutput, /ttl_clause: r#" TTL partition_time \+ INTERVAL 30 DAY"#/);
    assert.match(replicatedModOutput, /settings_clause: r#" SETTINGS index_granularity = 8192"#/);
    assert.match(
        replicatedOutput,
        /ClickHouseColumnSpec::with_ddl_type\("amount", r#"UInt64"#, r#"UInt64 CODEC\(ZSTD\(3\)\)"#\)/,
    );

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
    const distributedOutput = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [],
        hasAnchorEvents: false,
        hasClickHouseInstructionTypes: false,
        clickHouseDdl: distributedDdl,
        clickHouseEventDdl: getClickHouseDdlContext(true, 'event'),
    });

    assert.match(distributedOutput, /local_engine: Some\(r#"ReplicatedMergeTree/);
    assert.match(distributedOutput, /local_table_suffix: r#"_local"#/);
    assert.match(
        distributedOutput,
        /Distributed\('prod_cluster', 'analytics', '\{table_name\}_local', cityHash64\(instruction_id\)\)/,
    );

    const mergeTreeDdl = getClickHouseDdlContext(
        {
            ddlMode: 'merge-tree',
            engineSettings: { index_granularity: 8192 },
        },
        'instruction',
    );
    const mergeTreeOutput = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [],
        hasAnchorEvents: false,
        hasClickHouseInstructionTypes: false,
        clickHouseDdl: mergeTreeDdl,
        clickHouseEventDdl: getClickHouseDdlContext(true, 'event'),
    });

    assert.match(
        mergeTreeOutput,
        /settings_clause: r#" SETTINGS index_granularity = 8192, non_replicated_deduplication_window = 1000"#/,
    );

    const customEventDdl = getClickHouseDdlContext(
        {
            orderBy: { event: ['event_id', 'slot'] },
        },
        'event',
    );
    const customEventOutput = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [{ name: 'swapEvent' }],
        hasAnchorEvents: true,
        hasClickHouseInstructionTypes: false,
        clickHouseDdl: defaultInstructionDdl,
        clickHouseEventDdl: customEventDdl,
    });

    assert.match(customEventOutput, /order_by: r#"\(event_id, slot\)"#/);
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
    const tokenProgramAccountsMod = render('accountsMod.njk', {
        program: { name: 'tokenProgram' },
        originalProgramName: 'token-program',
        accountsToExport: [{ name: 'mint' }, { name: 'token' }, { name: 'multisig' }],
        withPostgres: true,
        withGraphQL: true,
        withClickHouse: true,
        postgresMode: 'typed',
        imports: 'use { crate::{TokenProgramDecoder, PROGRAM_ID}, solana_program_pack::Pack };',
    });

    assert.match(tokenProgramAccountsMod, /spl_token_interface::state::Mint::unpack\(&account\.data\)/);
    assert.match(tokenProgramAccountsMod, /spl_token_interface::state::Account::unpack\(&account\.data\)/);
    assert.match(tokenProgramAccountsMod, /spl_token_interface::state::Multisig::unpack\(&account\.data\)/);
    assert.match(tokenProgramAccountsMod, /decode_token_account_uses_spl_token_pack_layout/);
    assert.doesNotMatch(tokenProgramAccountsMod, /::decode\(data\)/);

    const tokenProgramTokenPage = render('accountsPage.njk', {
        account: { name: 'token' },
        originalProgramName: 'token-program',
        imports: 'use { crate::types::AccountState, solana_pubkey::Pubkey };',
        typeManifest: {
            type: `{
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub delegate: Option<Pubkey>,
    pub state: AccountState,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<Pubkey>,
}`,
        },
    });

    assert.match(tokenProgramTokenPage, /impl From<spl_token_interface::state::Account> for Token/);
    assert.match(tokenProgramTokenPage, /value\.delegate\.into\(\)/);
    assert.match(tokenProgramTokenPage, /spl_token_interface::state::AccountState::Initialized/);
    assert.doesNotMatch(tokenProgramTokenPage, /BorshDeserialize::deserialize/);

    const cargoTomlGenerator = fs.readFileSync('src/cargoTomlGenerator.ts', 'utf8');
    assert.match(cargoTomlGenerator, /isTokenProgram/);
    assert.match(cargoTomlGenerator, /'spl-token-interface'/);

    const tokenProgramAccountsModFromPackageName = render('accountsMod.njk', {
        program: { name: 'token-program' },
        originalProgramName: 'token',
        packageName: 'token-program',
        accountsToExport: [{ name: 'mint' }, { name: 'token' }, { name: 'multisig' }],
        withPostgres: true,
        withGraphQL: true,
        withClickHouse: true,
        postgresMode: 'typed',
        imports: 'use { crate::{TokenProgramDecoder, PROGRAM_ID}, solana_program_pack::Pack };',
    });

    assert.match(
        tokenProgramAccountsModFromPackageName,
        /spl_token_interface::state::Account::unpack\(&account\.data\)/,
    );
    assert.doesNotMatch(tokenProgramAccountsModFromPackageName, /::decode\(data\)/);
}

{
    const mapper = new ClickHouseRowMapper({ getDefinedTypesMap: () => new Map() });
    const plan = mapper.planType(
        {
            kind: 'structTypeNode',
            fields: [
                {
                    name: 'owner',
                    type: { kind: 'publicKeyTypeNode' },
                },
            ],
        },
        [],
        [],
        new Set(),
        {
            reservedNames: ['owner'],
            collisionPrefix: 'token',
        },
    );

    assert.equal(plan.fields.length, 1);
    assert.equal(plan.fields[0].column, 'token_owner');
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
    const tupleVariant = (name, items) => ({
        kind: 'enumTupleVariantTypeNode',
        name,
        tuple: { kind: 'tupleTypeNode', items },
    });

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
