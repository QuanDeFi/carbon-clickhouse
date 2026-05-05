import { isNode, pascalCase, resolveNestedTypeNode, snakeCase, SnakeCaseString, TypeNode } from '@codama/nodes';

export type ClickHouseFlattenedField = {
    column: string;
    rustPath: string;
    rowType: string;
    clickHouseColumnType: string;
    expr: string;
    docs: string[];
};

export type ClickHouseRowPlan = {
    fields: ClickHouseFlattenedField[];
    helperDefinitions: string[];
};

type ClickHouseMappedValue = {
    rowType: string;
    clickHouseColumnType: string;
    expr: string;
    defaultExpr: string;
    nullableSafe: boolean;
};

type ClickHouseFieldSpec = {
    name: string;
    rowType: string;
    clickHouseColumnType: string;
    expr: string;
    defaultExpr: string;
};

type HelperContext = {
    helpers: Map<string, string>;
    emitting: Set<string>;
};

type PayloadField = {
    variantName: string;
    variantRustName: string;
    sourceName: string;
    bindingName: string;
    typeNode: TypeNode;
    baseTypeNode: TypeNode;
    sourceOptional: boolean;
};

type PayloadFieldGroup = {
    outputName: string;
    typeNode: TypeNode;
    entries: PayloadField[];
};

const UNSIGNED_ORDER = ['u8', 'u16', 'u32', 'u64', 'u128'];
const SIGNED_ORDER = ['i8', 'i16', 'i32', 'i64', 'i128'];

export class ClickHouseRowMapper {
    constructor(
        private ctx: {
            getDefinedTypesMap: () => Map<string, any> | null;
        },
    ) {}

    planType(typeNode: TypeNode, prefix: string[], docsPrefix: string[], seen: Set<string>): ClickHouseRowPlan {
        const helperContext: HelperContext = { helpers: new Map(), emitting: new Set() };
        const fields = this.flattenTypeWithContext(typeNode, prefix, docsPrefix, seen, helperContext);
        return {
            fields,
            helperDefinitions: [...helperContext.helpers.values()],
        };
    }

    flattenType(typeNode: TypeNode, prefix: string[], docsPrefix: string[], seen: Set<string>): ClickHouseFlattenedField[] {
        return this.planType(typeNode, prefix, docsPrefix, seen).fields;
    }

    private flattenTypeWithContext(
        typeNode: TypeNode,
        prefix: string[],
        docsPrefix: string[],
        seen: Set<string>,
        helperContext: HelperContext,
    ): ClickHouseFlattenedField[] {
        const out: ClickHouseFlattenedField[] = [];

        const makeName = (nameParts: string[]): string => {
            let col = snakeCase(nameParts.join('_'));
            if (seen.has(col)) {
                let i = 1;
                while (seen.has(`${col}_${i}`)) i++;
                col = `${col}_${i}` as SnakeCaseString;
            }
            seen.add(col);
            return col;
        };

        const unwrapped = this.unwrapTransparent(typeNode);

        if (isNode(unwrapped, 'structTypeNode')) {
            for (const field of unwrapped.fields) {
                out.push(
                    ...this.flattenTypeWithContext(
                        field.type,
                        [...prefix, snakeCase(field.name)],
                        [],
                        seen,
                        helperContext,
                    ),
                );
            }
            return out;
        }

        if (this.isOptionNode(unwrapped)) {
            const item = (unwrapped as any).item as TypeNode;
            const sourceField = `source.${prefix.join('.')}`;
            const specs = this.optionFieldSpecs(makeName(prefix), item, sourceField, helperContext);
            return specs.map(spec => ({
                column: spec.name,
                rustPath: prefix.join('.'),
                rowType: spec.rowType,
                clickHouseColumnType: spec.clickHouseColumnType,
                docs: docsPrefix,
                expr: spec.expr,
            }));
        }

        const column = makeName(prefix);
        const sourceField = `source.${prefix.join('.')}`;
        const mapped = this.mapValue(unwrapped, sourceField, helperContext);
        return [
            {
                column,
                rustPath: prefix.join('.'),
                rowType: mapped.rowType,
                clickHouseColumnType: mapped.clickHouseColumnType,
                docs: docsPrefix,
                expr: mapped.expr,
            },
        ];
    }

    private mapValue(
        typeNode: TypeNode,
        source: string,
        helperContext: HelperContext,
        targetTypeNode?: TypeNode,
        sourceRef = false,
    ): ClickHouseMappedValue {
        const unwrapped = this.unwrapTransparent(typeNode);
        const target = targetTypeNode ? this.unwrapTransparent(targetTypeNode) : undefined;

        if (isNode(unwrapped, 'booleanTypeNode')) {
            return {
                rowType: 'bool',
                clickHouseColumnType: 'Bool',
                expr: sourceRef ? `*${source}` : source,
                defaultExpr: 'false',
                nullableSafe: true,
            };
        }

        if (isNode(unwrapped, 'bytesTypeNode')) {
            return {
                rowType: 'Vec<u8>',
                clickHouseColumnType: 'Array(UInt8)',
                expr: `${source}.to_vec()`,
                defaultExpr: 'Vec::new()',
                nullableSafe: false,
            };
        }

        if (isNode(unwrapped, 'stringTypeNode')) {
            return {
                rowType: 'String',
                clickHouseColumnType: 'String',
                expr: `${source}.clone()`,
                defaultExpr: 'String::new()',
                nullableSafe: true,
            };
        }

        if (isNode(unwrapped, 'publicKeyTypeNode')) {
            return {
                rowType: 'String',
                clickHouseColumnType: 'String',
                expr: `${source}.to_string()`,
                defaultExpr: 'String::new()',
                nullableSafe: true,
            };
        }

        if (isNode(unwrapped, 'numberTypeNode')) {
            const targetFormat = target && isNode(target, 'numberTypeNode') ? String((target as any).format) : undefined;
            return this.mapNumber(String((unwrapped as any).format), source, helperContext, targetFormat, sourceRef);
        }

        if (isNode(unwrapped, 'fixedSizeTypeNode')) {
            const innerType = unwrapped.type;
            if (isNode(innerType, 'bytesTypeNode')) {
                return {
                    rowType: 'Vec<u8>',
                    clickHouseColumnType: 'Array(UInt8)',
                    expr: `${source}.to_vec()`,
                    defaultExpr: 'Vec::new()',
                    nullableSafe: false,
                };
            }
            const inner = this.mapValue(innerType, 'value', helperContext, undefined, true);
            return {
                rowType: `Vec<${inner.rowType}>`,
                clickHouseColumnType: `Array(${inner.clickHouseColumnType})`,
                expr: `${source}.iter().map(|value| ${inner.expr}).collect()`,
                defaultExpr: 'Vec::new()',
                nullableSafe: false,
            };
        }

        if (isNode(unwrapped, 'arrayTypeNode')) {
            const inner = this.mapValue(unwrapped.item, 'value', helperContext, undefined, true);
            return {
                rowType: `Vec<${inner.rowType}>`,
                clickHouseColumnType: `Array(${inner.clickHouseColumnType})`,
                expr: `${source}.iter().map(|value| ${inner.expr}).collect()`,
                defaultExpr: 'Vec::new()',
                nullableSafe: false,
            };
        }

        if (isNode(unwrapped, 'tupleTypeNode')) {
            if (unwrapped.items.length === 1) {
                return this.mapValue(unwrapped.items[0], source, helperContext, target, sourceRef);
            }
            return this.mapInlineTuple(unwrapped, source, helperContext);
        }

        if (isNode(unwrapped, 'definedTypeLinkNode')) {
            const resolved = this.resolveDefinedType(unwrapped.name);
            if (resolved?.kind === 'enumTypeNode') {
                return this.mapEnum(
                    resolved,
                    source,
                    helperContext,
                    pascalCase(unwrapped.name),
                    `crate::types::${pascalCase(unwrapped.name)}`,
                    sourceRef,
                );
            }
            if (resolved?.kind === 'structTypeNode') {
                return this.mapDefinedStruct(
                    resolved,
                    source,
                    helperContext,
                    pascalCase(unwrapped.name),
                    `crate::types::${pascalCase(unwrapped.name)}`,
                    sourceRef,
                );
            }
            if (resolved?.kind === 'tupleTypeNode') {
                if (resolved.items.length === 1) {
                    return this.mapValue(resolved.items[0], source, helperContext, target, sourceRef);
                }
                return this.mapDefinedTuple(
                    resolved,
                    source,
                    helperContext,
                    pascalCase(unwrapped.name),
                    `crate::types::${pascalCase(unwrapped.name)}`,
                    sourceRef,
                );
            }
            if (resolved) {
                return this.mapValue(resolved, source, helperContext, target, sourceRef);
            }
        }

        if (isNode(unwrapped, 'structTypeNode')) {
            return this.mapInlineStruct(unwrapped, source, helperContext);
        }

        if (isNode(unwrapped, 'enumTypeNode')) {
            return this.mapEnum(unwrapped, source, helperContext, this.helperNameFromSource(source), null, sourceRef);
        }

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: 'JSON',
            expr: `serde_json::to_value(${source}).expect("serialize clickhouse field")`,
            defaultExpr: 'serde_json::Value::Null',
            nullableSafe: false,
        };
    }

    private mapDefinedStruct(
        typeNode: TypeNode,
        source: string,
        helperContext: HelperContext,
        typeName: string,
        sourceType: string,
        sourceRef = false,
    ): ClickHouseMappedValue {
        const helperName = `ClickHouse${typeName}`;
        if (!helperContext.helpers.has(helperName) && !helperContext.emitting.has(helperName)) {
            helperContext.emitting.add(helperName);
            const fields = this.structFieldSpecs(typeNode, 'value', helperContext);
            const definition = this.renderStructHelper(helperName, sourceType, fields);
            helperContext.helpers.set(helperName, definition);
            helperContext.emitting.delete(helperName);
        }

        return {
            rowType: helperName,
            clickHouseColumnType: this.helperTupleType(helperName, typeNode, helperContext),
            expr: `${helperName}::from(${sourceRef ? source : `&${source}`})`,
            defaultExpr: `${helperName}::default()`,
            nullableSafe: false,
        };
    }

    private mapInlineStruct(typeNode: TypeNode, source: string, helperContext: HelperContext): ClickHouseMappedValue {
        const helperName = `ClickHouse${this.helperNameFromSource(source)}`;
        if (!helperContext.helpers.has(helperName) && !helperContext.emitting.has(helperName)) {
            helperContext.emitting.add(helperName);
            const fields = this.structFieldSpecs(typeNode, 'value', helperContext);
            const definition = this.renderStructHelper(helperName, null, fields);
            helperContext.helpers.set(helperName, definition);
            helperContext.emitting.delete(helperName);
        }

        return {
            rowType: helperName,
            clickHouseColumnType: this.helperTupleType(helperName, typeNode, helperContext),
            expr: this.renderStructLiteral(helperName, source, typeNode, helperContext),
            defaultExpr: `${helperName}::default()`,
            nullableSafe: false,
        };
    }

    private mapDefinedTuple(
        typeNode: TypeNode,
        source: string,
        helperContext: HelperContext,
        typeName: string,
        sourceType: string,
        sourceRef = false,
    ): ClickHouseMappedValue {
        const helperName = `ClickHouse${typeName}`;
        if (!helperContext.helpers.has(helperName) && !helperContext.emitting.has(helperName)) {
            helperContext.emitting.add(helperName);
            const fields = this.tupleFieldSpecs(typeNode, 'value', helperContext);
            const definition = this.renderStructHelper(helperName, sourceType, fields);
            helperContext.helpers.set(helperName, definition);
            helperContext.emitting.delete(helperName);
        }

        return {
            rowType: helperName,
            clickHouseColumnType: this.helperTupleType(helperName, typeNode, helperContext),
            expr: `${helperName}::from(${sourceRef ? source : `&${source}`})`,
            defaultExpr: `${helperName}::default()`,
            nullableSafe: false,
        };
    }

    private mapInlineTuple(typeNode: TypeNode, source: string, helperContext: HelperContext): ClickHouseMappedValue {
        const helperName = `ClickHouse${this.helperNameFromSource(source)}`;
        if (!helperContext.helpers.has(helperName) && !helperContext.emitting.has(helperName)) {
            helperContext.emitting.add(helperName);
            const fields = this.tupleFieldSpecs(typeNode, 'value', helperContext);
            const definition = this.renderStructHelper(helperName, null, fields);
            helperContext.helpers.set(helperName, definition);
            helperContext.emitting.delete(helperName);
        }

        return {
            rowType: helperName,
            clickHouseColumnType: this.helperTupleType(helperName, typeNode, helperContext),
            expr: this.renderStructLiteral(helperName, source, typeNode, helperContext),
            defaultExpr: `${helperName}::default()`,
            nullableSafe: false,
        };
    }

    private mapEnum(
        typeNode: TypeNode,
        source: string,
        helperContext: HelperContext,
        typeName: string,
        sourceType: string | null,
        sourceRef = false,
    ): ClickHouseMappedValue {
        if (!isNode(typeNode, 'enumTypeNode')) {
            return this.mapValue(typeNode, source, helperContext);
        }

        if (this.isFieldlessEnum(typeNode)) {
            return {
                rowType: 'String',
                clickHouseColumnType: this.enumClickHouseType(typeNode),
                expr: `clickhouse_enum_variant(${sourceRef ? source : `&${source}`})`,
                defaultExpr: `"${this.firstEnumVariantName(typeNode)}".to_string()`,
                nullableSafe: true,
            };
        }

        const helperName = `ClickHouse${typeName}`;
        if (!helperContext.helpers.has(helperName) && !helperContext.emitting.has(helperName)) {
            helperContext.emitting.add(helperName);
            const fields = this.payloadEnumFieldSpecs(typeNode, helperContext);
            const definition = this.renderPayloadEnumHelper(helperName, sourceType, typeNode, fields, helperContext);
            helperContext.helpers.set(helperName, definition);
            helperContext.emitting.delete(helperName);
        }

        return {
            rowType: helperName,
            clickHouseColumnType: this.payloadEnumTupleType(typeNode, helperContext),
            expr: `${helperName}::from(${sourceRef ? source : `&${source}`})`,
            defaultExpr: `${helperName}::default()`,
            nullableSafe: false,
        };
    }

    private structFieldSpecs(typeNode: TypeNode, sourceRoot: string, helperContext: HelperContext): ClickHouseFieldSpec[] {
        if (!isNode(typeNode, 'structTypeNode')) {
            const mapped = this.mapValue(typeNode, sourceRoot, helperContext);
            return [
                {
                    name: 'value',
                    rowType: mapped.rowType,
                    clickHouseColumnType: mapped.clickHouseColumnType,
                    expr: mapped.expr,
                    defaultExpr: mapped.defaultExpr,
                },
            ];
        }

        const fields: ClickHouseFieldSpec[] = [];
        for (const field of typeNode.fields) {
            const fieldName = snakeCase(field.name);
            const fieldSource = `${sourceRoot}.${fieldName}`;
            const unwrapped = this.unwrapTransparent(field.type);
            if (this.isOptionNode(unwrapped)) {
                fields.push(...this.optionFieldSpecs(fieldName, (unwrapped as any).item as TypeNode, fieldSource, helperContext));
            } else {
                const mapped = this.mapValue(unwrapped, fieldSource, helperContext);
                fields.push({
                    name: fieldName,
                    rowType: mapped.rowType,
                    clickHouseColumnType: mapped.clickHouseColumnType,
                    expr: mapped.expr,
                    defaultExpr: mapped.defaultExpr,
                });
            }
        }
        return fields;
    }

    private tupleFieldSpecs(typeNode: TypeNode, sourceRoot: string, helperContext: HelperContext): ClickHouseFieldSpec[] {
        if (!isNode(typeNode, 'tupleTypeNode')) {
            const mapped = this.mapValue(typeNode, sourceRoot, helperContext);
            return [
                {
                    name: 'value',
                    rowType: mapped.rowType,
                    clickHouseColumnType: mapped.clickHouseColumnType,
                    expr: mapped.expr,
                    defaultExpr: mapped.defaultExpr,
                },
            ];
        }

        return typeNode.items.flatMap((item, index) => {
            const fieldName = `value_${index}`;
            const fieldSource = `${sourceRoot}.${index}`;
            const unwrapped = this.unwrapTransparent(item);
            if (this.isOptionNode(unwrapped)) {
                return this.optionFieldSpecs(fieldName, (unwrapped as any).item as TypeNode, fieldSource, helperContext);
            }

            const mapped = this.mapValue(unwrapped, fieldSource, helperContext);
            return [
                {
                    name: fieldName,
                    rowType: mapped.rowType,
                    clickHouseColumnType: mapped.clickHouseColumnType,
                    expr: mapped.expr,
                    defaultExpr: mapped.defaultExpr,
                },
            ];
        });
    }

    private optionFieldSpecs(
        fieldName: string,
        item: TypeNode,
        source: string,
        helperContext: HelperContext,
    ): ClickHouseFieldSpec[] {
        const mapped = this.mapValue(item, 'value', helperContext, undefined, true);
        if (mapped.nullableSafe) {
            return [
                {
                    name: fieldName,
                    rowType: `Option<${mapped.rowType}>`,
                    clickHouseColumnType: `Nullable(${mapped.clickHouseColumnType})`,
                    expr: `${source}.as_ref().map(|value| ${mapped.expr})`,
                    defaultExpr: 'None',
                },
            ];
        }

        return [
            {
                name: `${fieldName}_present`,
                rowType: 'bool',
                clickHouseColumnType: 'Bool',
                expr: `${source}.is_some()`,
                defaultExpr: 'false',
            },
            {
                name: fieldName,
                rowType: mapped.rowType,
                clickHouseColumnType: mapped.clickHouseColumnType,
                expr: `${source}.as_ref().map(|value| ${mapped.expr}).unwrap_or_default()`,
                defaultExpr: mapped.defaultExpr,
            },
        ];
    }

    private variantUnionFieldSpecs(group: PayloadFieldGroup, helperContext: HelperContext): ClickHouseFieldSpec[] {
        const mapped = this.mapValue(group.typeNode, 'value', helperContext);
        if (mapped.nullableSafe) {
            return [
                {
                    name: group.outputName,
                    rowType: `Option<${mapped.rowType}>`,
                    clickHouseColumnType: `Nullable(${mapped.clickHouseColumnType})`,
                    expr: 'None',
                    defaultExpr: 'None',
                },
            ];
        }

        return [
            {
                name: `${group.outputName}_present`,
                rowType: 'bool',
                clickHouseColumnType: 'Bool',
                expr: 'false',
                defaultExpr: 'false',
            },
            {
                name: group.outputName,
                rowType: mapped.rowType,
                clickHouseColumnType: mapped.clickHouseColumnType,
                expr: mapped.defaultExpr,
                defaultExpr: mapped.defaultExpr,
            },
        ];
    }

    private payloadEnumFieldSpecs(typeNode: TypeNode, helperContext: HelperContext): ClickHouseFieldSpec[] {
        const groups = this.payloadFieldGroups(typeNode);
        return groups.flatMap(group => this.variantUnionFieldSpecs(group, helperContext));
    }

    private payloadFieldGroups(typeNode: TypeNode): PayloadFieldGroup[] {
        if (!isNode(typeNode, 'enumTypeNode')) return [];

        const fieldsByName = new Map<string, PayloadField[]>();
        for (const variant of typeNode.variants) {
            const variantName = String((variant as any).name);
            const variantRustName = pascalCase(variantName);
            const payloadFields = this.variantPayloadFields(variant as any, variantName, variantRustName);
            for (const field of payloadFields) {
                const fields = fieldsByName.get(field.sourceName) ?? [];
                fields.push(field);
                fieldsByName.set(field.sourceName, fields);
            }
        }

        const groups: PayloadFieldGroup[] = [];
        for (const [name, fields] of fieldsByName) {
            const promoted = this.promoteCompatibleTypes(fields.map(field => field.baseTypeNode));
            if (promoted) {
                groups.push({ outputName: name, typeNode: promoted, entries: fields });
                continue;
            }

            for (const field of fields) {
                groups.push({
                    outputName: `${snakeCase(field.variantName)}_${field.sourceName}`,
                    typeNode: field.baseTypeNode,
                    entries: [field],
                });
            }
        }

        return groups;
    }

    private variantPayloadFields(variant: any, variantName: string, variantRustName: string): PayloadField[] {
        if (variant.kind === 'enumEmptyVariantTypeNode') {
            return [];
        }

        if (variant.kind === 'enumStructVariantTypeNode') {
            const struct = this.unwrapTransparent(variant.struct as TypeNode);
            if (!isNode(struct, 'structTypeNode')) return [];
            return struct.fields.map(field => {
                const sourceName = snakeCase(field.name);
                const { baseTypeNode, sourceOptional } = this.basePayloadType(field.type);
                return {
                    variantName,
                    variantRustName,
                    sourceName,
                    bindingName: sourceName,
                    typeNode: field.type,
                    baseTypeNode,
                    sourceOptional,
                };
            });
        }

        if (variant.kind === 'enumTupleVariantTypeNode') {
            const tuple = this.unwrapTransparent(variant.tuple as TypeNode);
            if (!isNode(tuple, 'tupleTypeNode')) return [];
            return tuple.items.map((item, index) => {
                const sourceName = `value_${index}`;
                const { baseTypeNode, sourceOptional } = this.basePayloadType(item);
                return {
                    variantName,
                    variantRustName,
                    sourceName,
                    bindingName: sourceName,
                    typeNode: item,
                    baseTypeNode,
                    sourceOptional,
                };
            });
        }

        return [];
    }

    private renderStructHelper(helperName: string, sourceType: string | null, fields: ClickHouseFieldSpec[]): string {
        const fieldDefs = fields.map(field => `    pub ${field.name}: ${field.rowType},`).join('\n');
        const defaults = fields.map(field => `            ${field.name}: ${field.defaultExpr},`).join('\n');
        const fromImpl = sourceType
            ? `
impl From<&${sourceType}> for ${helperName} {
    fn from(value: &${sourceType}) -> Self {
        Self {
${fields.map(field => `            ${field.name}: ${field.expr},`).join('\n')}
        }
    }
}`
            : '';

        return `#[derive(Debug, Clone, serde::Serialize)]
pub struct ${helperName} {
${fieldDefs}
}

impl Default for ${helperName} {
    fn default() -> Self {
        Self {
${defaults}
        }
    }
}
${fromImpl}`;
    }

    private renderStructLiteral(
        helperName: string,
        source: string,
        typeNode: TypeNode,
        helperContext: HelperContext,
    ): string {
        const fields = isNode(typeNode, 'tupleTypeNode')
            ? this.tupleFieldSpecs(typeNode, source, helperContext)
            : this.structFieldSpecs(typeNode, source, helperContext);
        return `${helperName} {
${fields.map(field => `                    ${field.name}: ${field.expr},`).join('\n')}
                }`;
    }

    private renderPayloadEnumHelper(
        helperName: string,
        sourceType: string | null,
        typeNode: TypeNode,
        fields: ClickHouseFieldSpec[],
        helperContext: HelperContext,
    ): string {
        const firstVariant = this.firstEnumVariantName(typeNode);
        const fieldDefs = fields.map(field => `    pub ${field.name}: ${field.rowType},`).join('\n');
        const defaults = fields.map(field => `            ${field.name}: ${field.defaultExpr},`).join('\n');
        const fromImpl =
            sourceType && isNode(typeNode, 'enumTypeNode')
                ? `
impl From<&${sourceType}> for ${helperName} {
    fn from(value: &${sourceType}) -> Self {
        let mut row = match value {
${typeNode.variants
    .map(
        (variant: any) =>
            `            ${sourceType}::${pascalCase(String(variant.name))}${this.variantTagPattern(variant)} => Self::new("${pascalCase(String(variant.name))}".to_string()),`,
    )
    .join('\n')}
        };

        let _ = &mut row;
        match value {
${typeNode.variants
    .map((variant: any) => this.variantAssignmentArm(sourceType, variant, helperContext, this.payloadFieldGroups(typeNode)))
    .join('\n')}
        }

        row
    }
}`
                : '';

        return `#[derive(Debug, Clone, serde::Serialize)]
pub struct ${helperName} {
    pub variant: String,
${fieldDefs}
}

impl ${helperName} {
    fn new(variant: String) -> Self {
        Self {
            variant,
${defaults}
        }
    }
}

impl Default for ${helperName} {
    fn default() -> Self {
        Self::new("${firstVariant}".to_string())
    }
}
${fromImpl}`;
    }

    private variantPattern(variant: any): string {
        if (variant.kind === 'enumEmptyVariantTypeNode') return '';
        if (variant.kind === 'enumStructVariantTypeNode') {
            const struct = this.unwrapTransparent(variant.struct as TypeNode);
            if (!isNode(struct, 'structTypeNode') || struct.fields.length === 0) return ' { }';
            return ` { ${struct.fields.map(field => snakeCase(field.name)).join(', ')} }`;
        }
        if (variant.kind === 'enumTupleVariantTypeNode') {
            const tuple = this.unwrapTransparent(variant.tuple as TypeNode);
            if (!isNode(tuple, 'tupleTypeNode') || tuple.items.length === 0) return '()';
            return `(${tuple.items.map((_, index) => `value_${index}`).join(', ')})`;
        }
        return '';
    }

    private variantTagPattern(variant: any): string {
        if (variant.kind === 'enumEmptyVariantTypeNode') return '';
        if (variant.kind === 'enumStructVariantTypeNode') return ' { .. }';
        if (variant.kind === 'enumTupleVariantTypeNode') {
            const tuple = this.unwrapTransparent(variant.tuple as TypeNode);
            if (!isNode(tuple, 'tupleTypeNode') || tuple.items.length === 0) return '()';
            return '(..)';
        }
        return '';
    }

    private variantAssignmentArm(
        sourceType: string,
        variant: any,
        helperContext: HelperContext,
        allGroups: PayloadFieldGroup[],
    ): string {
        const variantName = String(variant.name);
        const pattern = `${sourceType}::${pascalCase(variantName)}${this.variantPattern(variant)}`;
        const groups = this.payloadFieldGroups({
            kind: 'enumTypeNode',
            variants: [variant],
            size: { kind: 'numberTypeNode', format: 'u8', endian: 'le' },
        } as any);
        const assignments: string[] = [];
        const singleVariantFields = groups.flatMap(group => group.entries);

        for (const field of singleVariantFields) {
            const group = this.findPayloadGroup(allGroups, field);
            if (!group) continue;
            assignments.push(...this.assignmentForPayloadField(group, field, helperContext));
        }

        if (assignments.length === 0) {
            return `            ${pattern} => {}`;
        }

        return `            ${pattern} => {
${assignments.map(line => `                ${line}`).join('\n')}
            }`;
    }

    private findPayloadGroup(groups: PayloadFieldGroup[], field: PayloadField): PayloadFieldGroup | null {
        return (
            groups.find(group =>
                group.entries.some(
                    entry =>
                        entry.variantName === field.variantName &&
                        entry.sourceName === field.sourceName &&
                        entry.bindingName === field.bindingName,
                ),
            ) ?? null
        );
    }

    private assignmentForPayloadField(
        group: PayloadFieldGroup,
        field: PayloadField,
        helperContext: HelperContext,
    ): string[] {
        const mapped = this.mapValue(field.baseTypeNode, field.bindingName, helperContext, group.typeNode, true);
        const optionalMapped = this.mapValue(field.baseTypeNode, 'value', helperContext, group.typeNode, true);
        const specs = this.variantUnionFieldSpecs(group, helperContext);

        if (specs.length === 1) {
            if (field.sourceOptional) {
                return [`row.${specs[0].name} = ${field.bindingName}.as_ref().map(|value| ${optionalMapped.expr});`];
            }
            return [`row.${specs[0].name} = Some(${mapped.expr});`];
        }

        const presentName = specs[0].name;
        const valueName = specs[1].name;
        if (field.sourceOptional) {
            return [
                `if let Some(value) = ${field.bindingName}.as_ref() {`,
                `    row.${presentName} = true;`,
                `    row.${valueName} = ${optionalMapped.expr};`,
                `}`,
            ];
        }

        return [`row.${presentName} = true;`, `row.${valueName} = ${mapped.expr};`];
    }

    private helperTupleType(helperName: string, typeNode: TypeNode, helperContext: HelperContext): string {
        const helper = helperContext.helpers.get(helperName);
        const fields = isNode(typeNode, 'tupleTypeNode')
            ? this.tupleFieldSpecs(typeNode, 'value', helperContext)
            : this.structFieldSpecs(typeNode, 'value', helperContext);
        if (!helper && helperContext.emitting.has(helperName)) {
            return `Tuple(${fields.map(field => `${field.name} ${field.clickHouseColumnType}`).join(', ')})`;
        }
        return `Tuple(${fields.map(field => `${field.name} ${field.clickHouseColumnType}`).join(', ')})`;
    }

    private payloadEnumTupleType(typeNode: TypeNode, helperContext: HelperContext): string {
        const fields = this.payloadEnumFieldSpecs(typeNode, helperContext);
        const variantType = this.enumClickHouseType(typeNode);
        return `Tuple(variant ${variantType}${fields.length > 0 ? ', ' : ''}${fields
            .map(field => `${field.name} ${field.clickHouseColumnType}`)
            .join(', ')})`;
    }

    private basePayloadType(typeNode: TypeNode): { baseTypeNode: TypeNode; sourceOptional: boolean } {
        const unwrapped = this.unwrapTransparent(typeNode);
        if (this.isOptionNode(unwrapped)) {
            return { baseTypeNode: (unwrapped as any).item as TypeNode, sourceOptional: true };
        }
        return { baseTypeNode: unwrapped, sourceOptional: false };
    }

    private promoteCompatibleTypes(typeNodes: TypeNode[]): TypeNode | null {
        const unwrapped = typeNodes.map(typeNode => this.unwrapTransparent(typeNode));
        const signatures = new Set(unwrapped.map(typeNode => this.typeSignature(typeNode)));
        if (signatures.size === 1) return unwrapped[0];

        const numberFormats = unwrapped
            .filter(typeNode => isNode(typeNode, 'numberTypeNode'))
            .map(typeNode => String((typeNode as any).format));
        if (numberFormats.length === unwrapped.length) {
            const unsigned = this.promoteNumberFormats(numberFormats, UNSIGNED_ORDER);
            if (unsigned) return { kind: 'numberTypeNode', format: unsigned, endian: 'le' } as any;

            const signed = this.promoteNumberFormats(numberFormats, SIGNED_ORDER);
            if (signed) return { kind: 'numberTypeNode', format: signed, endian: 'le' } as any;
        }

        return null;
    }

    private promoteNumberFormats(formats: string[], order: string[]): string | null {
        if (!formats.every(format => order.includes(format))) return null;
        return formats.reduce((max, format) => (order.indexOf(format) > order.indexOf(max) ? format : max), formats[0]);
    }

    private typeSignature(typeNode: TypeNode): string {
        const unwrapped = this.unwrapTransparent(typeNode);
        if (isNode(unwrapped, 'definedTypeLinkNode')) return `defined:${String(unwrapped.name)}`;
        if (isNode(unwrapped, 'numberTypeNode')) return `number:${String((unwrapped as any).format)}`;
        if (isNode(unwrapped, 'arrayTypeNode')) return `array:${this.typeSignature(unwrapped.item)}`;
        if (isNode(unwrapped, 'fixedSizeTypeNode')) return `fixed:${unwrapped.size}:${this.typeSignature(unwrapped.type)}`;
        if (isNode(unwrapped, 'optionTypeNode') || isNode(unwrapped, 'zeroableOptionTypeNode') || isNode(unwrapped, 'remainderOptionTypeNode')) {
            return `option:${this.typeSignature((unwrapped as any).item as TypeNode)}`;
        }
        if (isNode(unwrapped, 'structTypeNode')) {
            return `struct:${unwrapped.fields.map(field => `${snakeCase(field.name)}:${this.typeSignature(field.type)}`).join(';')}`;
        }
        if (isNode(unwrapped, 'tupleTypeNode')) {
            return `tuple:${unwrapped.items.map(item => this.typeSignature(item)).join(';')}`;
        }
        return unwrapped.kind;
    }

    private isOptionNode(typeNode: TypeNode): boolean {
        return (
            isNode(typeNode, 'optionTypeNode') ||
            isNode(typeNode, 'zeroableOptionTypeNode') ||
            isNode(typeNode, 'remainderOptionTypeNode')
        );
    }

    private unwrapTransparent(typeNode: TypeNode): TypeNode {
        let current = typeNode;
        while (
            isNode(current, 'hiddenPrefixTypeNode') ||
            isNode(current, 'hiddenSuffixTypeNode') ||
            isNode(current, 'sizePrefixTypeNode') ||
            isNode(current, 'preOffsetTypeNode') ||
            isNode(current, 'postOffsetTypeNode') ||
            isNode(current, 'sentinelTypeNode')
        ) {
            current = (current as any).type as TypeNode;
        }
        return current;
    }

    private isFieldlessEnum(typeNode: TypeNode): boolean {
        return isNode(typeNode, 'enumTypeNode') && typeNode.variants.every(variant => variant.kind === 'enumEmptyVariantTypeNode');
    }

    private enumClickHouseType(typeNode: TypeNode): string {
        if (!isNode(typeNode, 'enumTypeNode')) {
            return 'LowCardinality(String)';
        }

        const enumType = typeNode.variants.length <= 128 ? 'Enum8' : 'Enum16';
        const values = typeNode.variants
            .map((variant, index) => `'${this.escapeClickHouseEnumValue(pascalCase(String((variant as any).name)))}' = ${index}`)
            .join(', ');
        return `${enumType}(${values})`;
    }

    private firstEnumVariantName(typeNode: TypeNode): string {
        if (!isNode(typeNode, 'enumTypeNode') || typeNode.variants.length === 0) return '';
        return pascalCase(String((typeNode.variants[0] as any).name));
    }

    private escapeClickHouseEnumValue(value: string): string {
        return value.replace(/\\/g, '\\\\').replace(/'/g, "\\'");
    }

    private mapNumber(
        format: string,
        source: string,
        helperContext: HelperContext,
        targetFormat?: string,
        sourceRef = false,
    ): ClickHouseMappedValue {
        const finalFormat = targetFormat ?? format;
        const sourceValue = sourceRef ? `*${source}` : source;
        const valueExpr = targetFormat && targetFormat !== format ? `(${sourceValue}) as ${targetFormat}` : sourceValue;

        switch (finalFormat) {
            case 'u8':
                return { rowType: 'u8', clickHouseColumnType: 'UInt8', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'u16':
                return { rowType: 'u16', clickHouseColumnType: 'UInt16', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'u32':
                return { rowType: 'u32', clickHouseColumnType: 'UInt32', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'u64':
                return { rowType: 'u64', clickHouseColumnType: 'UInt64', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'u128':
                this.ensureWideIntHelper(helperContext, 'ClickHouseUInt128', 'u128');
                return {
                    rowType: 'ClickHouseUInt128',
                    clickHouseColumnType: 'UInt128',
                    expr: `ClickHouseUInt128(${valueExpr})`,
                    defaultExpr: 'ClickHouseUInt128::default()',
                    nullableSafe: true,
                };
            case 'i8':
                return { rowType: 'i8', clickHouseColumnType: 'Int8', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'i16':
                return { rowType: 'i16', clickHouseColumnType: 'Int16', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'i32':
                return { rowType: 'i32', clickHouseColumnType: 'Int32', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'i64':
                return { rowType: 'i64', clickHouseColumnType: 'Int64', expr: valueExpr, defaultExpr: '0', nullableSafe: true };
            case 'i128':
                this.ensureWideIntHelper(helperContext, 'ClickHouseInt128', 'i128');
                return {
                    rowType: 'ClickHouseInt128',
                    clickHouseColumnType: 'Int128',
                    expr: `ClickHouseInt128(${valueExpr})`,
                    defaultExpr: 'ClickHouseInt128::default()',
                    nullableSafe: true,
                };
            case 'f32':
                return { rowType: 'f32', clickHouseColumnType: 'Float32', expr: valueExpr, defaultExpr: '0.0', nullableSafe: true };
            case 'f64':
                return { rowType: 'f64', clickHouseColumnType: 'Float64', expr: valueExpr, defaultExpr: '0.0', nullableSafe: true };
            default:
                return {
                    rowType: 'String',
                    clickHouseColumnType: 'String',
                    expr: `${source}.to_string()`,
                    defaultExpr: 'String::new()',
                    nullableSafe: true,
                };
        }
    }

    private ensureWideIntHelper(helperContext: HelperContext, helperName: string, rustType: string): void {
        if (helperContext.helpers.has(helperName)) return;
        helperContext.helpers.set(
            helperName,
            `#[derive(Debug, Clone, Copy, Default)]
pub struct ${helperName}(pub ${rustType});

impl serde::Serialize for ${helperName} {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}`,
        );
    }

    private resolveDefinedType(name: string): TypeNode | null {
        const definedTypesMap = this.ctx.getDefinedTypesMap();
        const definedType = definedTypesMap?.get(name);
        if (definedType?.type) {
            return definedType.type as TypeNode;
        }
        try {
            return resolveNestedTypeNode({ kind: 'definedTypeLinkNode', name } as any) as TypeNode;
        } catch {
            return null;
        }
    }

    private helperNameFromSource(source: string): string {
        return pascalCase(source.replace(/[^a-zA-Z0-9]+/g, '_'));
    }
}
