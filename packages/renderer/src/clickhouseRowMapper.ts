import { isNode, pascalCase, resolveNestedTypeNode, snakeCase, SnakeCaseString, TypeNode } from '@codama/nodes';

export type ClickHouseFlattenedField = {
    column: string;
    rustPath: string;
    rowType: string;
    clickHouseColumnType: string;
    expr: string;
    docs: string[];
};

type ClickHouseMappedValue = {
    rowType: string;
    clickHouseColumnType: string;
    expr: string;
};

export class ClickHouseRowMapper {
    constructor(
        private ctx: {
            getDefinedTypesMap: () => Map<string, any> | null;
        },
    ) {}

    flattenType(typeNode: TypeNode, prefix: string[], docsPrefix: string[], seen: Set<string>): ClickHouseFlattenedField[] {
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

        if (isNode(typeNode, 'structTypeNode')) {
            for (const field of typeNode.fields) {
                out.push(...this.flattenType(field.type, [...prefix, snakeCase(field.name)], [], seen));
            }
            return out;
        }

        if (isNode(typeNode, 'hiddenPrefixTypeNode') || isNode(typeNode, 'sizePrefixTypeNode')) {
            return this.flattenType(typeNode.type, prefix, docsPrefix, seen);
        }

        if (
            isNode(typeNode, 'optionTypeNode') ||
            isNode(typeNode, 'zeroableOptionTypeNode') ||
            isNode(typeNode, 'remainderOptionTypeNode')
        ) {
            const item = (typeNode as any).item as TypeNode;
            const column = makeName(prefix);
            const inner = this.mapValue(item, 'value');
            const sourceField = `source.${prefix.join('.')}`;
            return [
                {
                    column,
                    rustPath: prefix.join('.'),
                    rowType: `Option<${inner.rowType}>`,
                    clickHouseColumnType: `Nullable(${inner.clickHouseColumnType})`,
                    docs: docsPrefix,
                    expr: `${sourceField}.map(|value| ${inner.expr})`,
                },
            ];
        }

        const column = makeName(prefix);
        const sourceField = `source.${prefix.join('.')}`;
        const mapped = this.mapValue(typeNode, sourceField);
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

    private mapValue(typeNode: TypeNode, source: string): ClickHouseMappedValue {
        if (isNode(typeNode, 'hiddenPrefixTypeNode') || isNode(typeNode, 'sizePrefixTypeNode')) {
            return this.mapValue(typeNode.type, source);
        }

        if (isNode(typeNode, 'booleanTypeNode')) {
            return { rowType: 'bool', clickHouseColumnType: 'Bool', expr: source };
        }

        if (isNode(typeNode, 'bytesTypeNode')) {
            return { rowType: 'Vec<u8>', clickHouseColumnType: 'Array(UInt8)', expr: `${source}.to_vec()` };
        }

        if (isNode(typeNode, 'stringTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: source };
        }

        if (isNode(typeNode, 'publicKeyTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: `${source}.to_string()` };
        }

        if (isNode(typeNode, 'numberTypeNode')) {
            return this.mapNumber((typeNode as any).format, source);
        }

        if (isNode(typeNode, 'fixedSizeTypeNode')) {
            const innerType = typeNode.type;
            if (isNode(innerType, 'bytesTypeNode')) {
                return { rowType: 'Vec<u8>', clickHouseColumnType: 'Array(UInt8)', expr: `${source}.to_vec()` };
            }
            const inner = this.mapCollectionItem(innerType, 'value');
            return {
                rowType: `Vec<${inner.rowType}>`,
                clickHouseColumnType: `Array(${inner.clickHouseColumnType})`,
                expr: `${source}.iter().map(|value| ${inner.expr}).collect()`,
            };
        }

        if (isNode(typeNode, 'arrayTypeNode')) {
            const inner = this.mapCollectionItem(typeNode.item, 'value');
            return {
                rowType: `Vec<${inner.rowType}>`,
                clickHouseColumnType: `Array(${inner.clickHouseColumnType})`,
                expr: `${source}.iter().map(|value| ${inner.expr}).collect()`,
            };
        }

        if (isNode(typeNode, 'tupleTypeNode') && typeNode.items.length === 1) {
            return this.mapValue(typeNode.items[0], source);
        }

        if (isNode(typeNode, 'definedTypeLinkNode')) {
            const resolved = this.resolveDefinedType(typeNode.name);
            if (resolved?.kind === 'enumTypeNode') {
                return this.mapEnum(resolved, `&${source}`);
            }
            if (resolved) {
                const simple = this.tryMapResolvedDefinedType(resolved, source);
                if (simple) {
                    return simple;
                }
            }
        }

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: 'JSON',
            expr: `serde_json::to_value(${source}).expect("serialize clickhouse field")`,
        };
    }

    private mapCollectionItem(typeNode: TypeNode, source: string): ClickHouseMappedValue {
        if (isNode(typeNode, 'hiddenPrefixTypeNode') || isNode(typeNode, 'sizePrefixTypeNode')) {
            return this.mapCollectionItem(typeNode.type, source);
        }

        if (isNode(typeNode, 'numberTypeNode') || isNode(typeNode, 'booleanTypeNode')) {
            return this.mapValue(typeNode, `*${source}`);
        }

        if (isNode(typeNode, 'stringTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: `${source}.clone()` };
        }

        if (isNode(typeNode, 'publicKeyTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: `${source}.to_string()` };
        }

        if (isNode(typeNode, 'bytesTypeNode')) {
            return { rowType: 'Vec<u8>', clickHouseColumnType: 'Array(UInt8)', expr: `${source}.to_vec()` };
        }

        if (isNode(typeNode, 'definedTypeLinkNode')) {
            const resolved = this.resolveDefinedType(typeNode.name);
            if (resolved?.kind === 'enumTypeNode') {
                return this.mapEnum(resolved, source);
            }
            if (resolved?.kind === 'structTypeNode') {
                return this.mapStructTuple(resolved, source);
            }
            if (resolved) {
                return this.mapCollectionItem(resolved, source);
            }
        }

        if (isNode(typeNode, 'structTypeNode')) {
            return this.mapStructTuple(typeNode, source);
        }

        if (isNode(typeNode, 'enumTypeNode')) {
            return this.mapEnum(typeNode, source);
        }

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: 'JSON',
            expr: `serde_json::to_value(${source}).expect("serialize clickhouse field")`,
        };
    }

    private mapStructTuple(typeNode: TypeNode, source: string): ClickHouseMappedValue {
        if (!isNode(typeNode, 'structTypeNode')) {
            return this.mapCollectionItem(typeNode, source);
        }

        const fields = typeNode.fields.map(field => {
            const fieldName = snakeCase(field.name);
            const fieldSource = `${source}.${fieldName}`;
            return {
                column: fieldName,
                mapped: this.mapTupleField(field.type, fieldSource),
            };
        });

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: `Tuple(${fields
                .map(field => `${field.column} ${field.mapped.clickHouseColumnType}`)
                .join(', ')})`,
            expr: `serde_json::json!({ ${fields
                .map(field => `"${field.column}": ${field.mapped.expr}`)
                .join(', ')} })`,
        };
    }

    private mapTupleField(typeNode: TypeNode, source: string): ClickHouseMappedValue {
        if (isNode(typeNode, 'hiddenPrefixTypeNode') || isNode(typeNode, 'sizePrefixTypeNode')) {
            return this.mapTupleField(typeNode.type, source);
        }

        if (isNode(typeNode, 'booleanTypeNode') || isNode(typeNode, 'numberTypeNode')) {
            const mapped = this.mapValue(typeNode, source);
            return { ...mapped, expr: source };
        }

        if (isNode(typeNode, 'stringTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: source };
        }

        if (isNode(typeNode, 'publicKeyTypeNode')) {
            return { rowType: 'String', clickHouseColumnType: 'String', expr: `${source}.to_string()` };
        }

        if (isNode(typeNode, 'bytesTypeNode')) {
            return { rowType: 'Vec<u8>', clickHouseColumnType: 'Array(UInt8)', expr: `${source}.to_vec()` };
        }

        if (isNode(typeNode, 'arrayTypeNode') || isNode(typeNode, 'fixedSizeTypeNode')) {
            return this.mapValue(typeNode, source);
        }

        if (isNode(typeNode, 'definedTypeLinkNode')) {
            const resolved = this.resolveDefinedType(typeNode.name);
            if (resolved?.kind === 'enumTypeNode') {
                return this.mapEnum(resolved, `&${source}`);
            }
            if (resolved?.kind === 'structTypeNode') {
                return this.mapStructTuple(resolved, source);
            }
            if (resolved) {
                return this.mapTupleField(resolved, source);
            }
        }

        if (isNode(typeNode, 'enumTypeNode')) {
            return this.mapEnum(typeNode, `&${source}`);
        }

        if (isNode(typeNode, 'structTypeNode')) {
            return this.mapStructTuple(typeNode, source);
        }

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: 'JSON',
            expr: `serde_json::to_value(${source}).expect("serialize clickhouse field")`,
        };
    }

    private mapEnum(typeNode: TypeNode, source: string): ClickHouseMappedValue {
        if (this.isFieldlessEnum(typeNode)) {
            return {
                rowType: 'String',
                clickHouseColumnType: this.enumClickHouseType(typeNode),
                expr: `clickhouse_enum_variant(${source})`,
            };
        }

        return {
            rowType: 'serde_json::Value',
            clickHouseColumnType: 'JSON',
            expr: `clickhouse_enum_json(${source})`,
        };
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

    private escapeClickHouseEnumValue(value: string): string {
        return value.replace(/\\/g, '\\\\').replace(/'/g, "\\'");
    }

    private mapNumber(format: string, source: string): ClickHouseMappedValue {
        switch (format) {
            case 'u8':
                return { rowType: 'u8', clickHouseColumnType: 'UInt8', expr: source };
            case 'u16':
                return { rowType: 'u16', clickHouseColumnType: 'UInt16', expr: source };
            case 'u32':
                return { rowType: 'u32', clickHouseColumnType: 'UInt32', expr: source };
            case 'u64':
                return { rowType: 'u64', clickHouseColumnType: 'UInt64', expr: source };
            case 'u128':
                return { rowType: 'u128', clickHouseColumnType: 'UInt128', expr: source };
            case 'i8':
                return { rowType: 'i8', clickHouseColumnType: 'Int8', expr: source };
            case 'i16':
                return { rowType: 'i16', clickHouseColumnType: 'Int16', expr: source };
            case 'i32':
                return { rowType: 'i32', clickHouseColumnType: 'Int32', expr: source };
            case 'i64':
                return { rowType: 'i64', clickHouseColumnType: 'Int64', expr: source };
            case 'i128':
                return { rowType: 'i128', clickHouseColumnType: 'Int128', expr: source };
            case 'f32':
                return { rowType: 'f32', clickHouseColumnType: 'Float32', expr: source };
            case 'f64':
                return { rowType: 'f64', clickHouseColumnType: 'Float64', expr: source };
            default:
                return { rowType: 'String', clickHouseColumnType: 'String', expr: `${source}.to_string()` };
        }
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

    private tryMapResolvedDefinedType(
        typeNode: TypeNode,
        source: string,
    ): ClickHouseMappedValue | null {
        if (
            isNode(typeNode, 'numberTypeNode') ||
            isNode(typeNode, 'booleanTypeNode') ||
            isNode(typeNode, 'bytesTypeNode') ||
            isNode(typeNode, 'stringTypeNode') ||
            isNode(typeNode, 'publicKeyTypeNode')
        ) {
            return this.mapValue(typeNode, source);
        }

        return null;
    }
}
