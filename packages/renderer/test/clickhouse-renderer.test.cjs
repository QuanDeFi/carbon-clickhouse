const assert = require('node:assert/strict');
const fs = require('node:fs');

const { camelCase, kebabCase, pascalCase, snakeCase, titleCase } = require('@codama/nodes');
const nunjucks = require('nunjucks');

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
const typedField = {
    column: 'amount',
    rowType: 'u64',
    clickHouseColumnType: 'UInt64',
    expr: 'source.amount',
};

function render(template, context) {
    return env.render(template, context);
}

{
    const output = render('instructionsClickHouseMod.njk', {
        program,
        instructionsToExport: [{ name: 'swap' }],
        events: [{ name: 'swapEvent' }],
        hasAnchorEvents: true,
    });

    assert.match(output, /pub mod swap_row;/);
    assert.match(output, /pub mod swap_event_event_row;/);
    assert.match(output, /SwapInstructionClickHouseRow/);
    assert.match(output, /SwapEventEventClickHouseRow/);
    assert.match(output, /ClickHouseRows<DemoProgramClickHouseInstructionRow>/);
    assert.match(output, /DemoProgramInstruction::Swap/);
    assert.match(output, /super::CpiEvent::SwapEvent/);
    assert.match(output, /create_table_sql\(swap_row::SwapInstructionClickHouseRow::DEFAULT_TABLE_NAME\)/);
}

{
    const output = render('accountsClickHouseMod.njk', {
        program,
        accountsToExport: [{ name: 'mint' }, { name: 'token' }],
    });

    assert.match(output, /pub mod mint_row;/);
    assert.match(output, /pub mod token_row;/);
    assert.match(output, /MintAccountClickHouseRow::create_table_sql/);
    assert.match(output, /TokenAccountClickHouseRow::create_table_sql/);
    assert.match(output, /ClickHouseAccountProcessor</);
    assert.match(output, /DemoProgramAccount::Mint/);
}

{
    const output = render('clickhouseRowPage.njk', {
        program,
        entityName: 'mint',
        isAccount: true,
        flatFields: [typedField],
    });

    assert.match(output, /pub struct MintAccountClickHouseRow/);
    assert.match(output, /deterministic_account_id/);
    assert.match(output, /pub amount: u64/);
    assert.match(output, /amount UInt64/);
    assert.match(output, /demo_program_mint_account_landing/);
}

{
    const output = render('eventInstructionClickHouseRowPage.njk', {
        program,
        event: { name: 'swapEvent' },
        flatFields: [typedField],
    });

    assert.match(output, /pub struct SwapEventEventClickHouseRow/);
    assert.match(output, /deterministic_event_id/);
    assert.match(output, /pub amount: u64/);
    assert.match(output, /amount UInt64/);
    assert.doesNotMatch(output, /data JSON/);
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
