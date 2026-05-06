#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

COMPILE_ALL=0
ALLOW_BROAD_CLICKHOUSE=0
SKIP_RENDERER=0
REGENERATE_IDL_DIR=""
REGENERATE_FROM_README=0
REGENERATE_STANDARD="anchor"
REGENERATE_RPC_URL=""
REGENERATE_LIMIT=0
SKIP_REGENERATED_COMPILE=0
NODE_BIN="${NODE_BIN:-node}"

usage() {
    cat <<'EOF'
Usage: scripts/validate-clickhouse-decoder-rollout.sh [options]

Non-committing ClickHouse rollout validation for decoder crates.

Options:
  --compile-all              Also run cargo check for every decoder without extra features.
  --allow-broad-clickhouse   Allow non-canary decoders with committed clickhouse feature/modules.
  --skip-renderer            Skip renderer test and type-check commands.
  --regenerate-idl-dir DIR   Generate ClickHouse-enabled decoders from local IDL JSON files into a temp dir.
  --regenerate-from-readme   Generate ClickHouse-enabled decoders from program IDs listed in README.md.
  --regenerate-standard STD  IDL standard for regeneration checks: anchor or codama. Default: anchor.
  --rpc-url URL              RPC URL for --regenerate-from-readme program-address IDL fetches.
  --regenerate-limit N       Limit regeneration checks to the first N discovered IDLs/program IDs.
  --skip-regenerated-compile Generate regenerated decoders but do not cargo check them.
  -h, --help                 Show this help.

Default behavior:
  - run renderer ClickHouse tests/type-check
  - scan every decoder crate
  - require committed ClickHouse output to remain canary-limited
  - cargo check decoder crates that currently expose a clickhouse feature

Regeneration checks are opt-in and write only to a temporary directory. They use
--with-clickhouse true and patch generated temporary crates to compile against
the local carbon-core path. The repository working tree must be unchanged after
the script exits. Regeneration runs the built CLI with NODE_BIN, defaulting to
node, and requires Node 20+ because current CLI dependencies require it.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --compile-all)
            COMPILE_ALL=1
            shift
            ;;
        --allow-broad-clickhouse)
            ALLOW_BROAD_CLICKHOUSE=1
            shift
            ;;
        --skip-renderer)
            SKIP_RENDERER=1
            shift
            ;;
        --regenerate-idl-dir)
            REGENERATE_IDL_DIR="${2:-}"
            if [[ -z "$REGENERATE_IDL_DIR" ]]; then
                echo "--regenerate-idl-dir requires a directory" >&2
                exit 2
            fi
            shift 2
            ;;
        --regenerate-from-readme)
            REGENERATE_FROM_README=1
            shift
            ;;
        --regenerate-standard)
            REGENERATE_STANDARD="${2:-}"
            if [[ "$REGENERATE_STANDARD" != "anchor" && "$REGENERATE_STANDARD" != "codama" ]]; then
                echo "--regenerate-standard must be anchor or codama" >&2
                exit 2
            fi
            shift 2
            ;;
        --rpc-url)
            REGENERATE_RPC_URL="${2:-}"
            if [[ -z "$REGENERATE_RPC_URL" ]]; then
                echo "--rpc-url requires a URL" >&2
                exit 2
            fi
            shift 2
            ;;
        --regenerate-limit)
            REGENERATE_LIMIT="${2:-}"
            if ! [[ "$REGENERATE_LIMIT" =~ ^[0-9]+$ ]]; then
                echo "--regenerate-limit requires a non-negative integer" >&2
                exit 2
            fi
            shift 2
            ;;
        --skip-regenerated-compile)
            SKIP_REGENERATED_COMPILE=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "unknown option: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ "$REGENERATE_FROM_README" -eq 1 && -z "$REGENERATE_RPC_URL" ]]; then
    echo "--regenerate-from-readme requires --rpc-url" >&2
    exit 2
fi

if [[ -n "$REGENERATE_IDL_DIR" || "$REGENERATE_FROM_README" -eq 1 ]]; then
    node_major="$("$NODE_BIN" -p 'Number(process.versions.node.split(".")[0])' 2>/dev/null || true)"
    if [[ -z "$node_major" || "$node_major" -lt 20 ]]; then
        echo "regeneration checks require Node 20+; set NODE_BIN to a compatible node binary" >&2
        exit 2
    fi
fi

before_status="$(git status --porcelain)"
temp_dir=""
cleanup() {
    if [[ -n "$temp_dir" && -d "$temp_dir" ]]; then
        rm -rf "$temp_dir"
    fi
}
trap cleanup EXIT

is_canary_decoder() {
    case "$1" in
        carbon-jupiter-swap-decoder|carbon-token-program-decoder)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

package_name() {
    sed -n 's/^name = "\(.*\)"/\1/p' "$1" | head -n 1
}

has_clickhouse_feature() {
    awk '
        /^\[features\]/ { in_features = 1; next }
        /^\[/ { in_features = 0 }
        in_features && /^clickhouse[[:space:]]*=/ { found = 1 }
        END { exit(found ? 0 : 1) }
    ' "$1"
}

has_clickhouse_module() {
    local decoder_dir="$1"
    find "$decoder_dir/src" -type d -name clickhouse -print -quit | grep -q .
}

decoder_name_from_package() {
    local package="$1"
    package="${package#carbon-}"
    package="${package%-decoder}"
    printf '%s\n' "$package"
}

readme_programs() {
    awk -F'|' '
        /^\| `carbon-/ {
            package = $2
            program = $4
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", package)
            gsub(/^[[:space:]]+|[[:space:]]+$/, "", program)
            gsub(/`/, "", package)
            if (program ~ /^[1-9A-HJ-NP-Za-km-z]{32,44}$/) {
                print package "|" program
            }
        }
    ' README.md
}

patch_generated_manifest_for_local_core() {
    local manifest="$1"
    local core_path="${ROOT_DIR}/crates/core"
    local test_utils_path="${ROOT_DIR}/crates/test-utils"

    perl -0pi -e "s#carbon-core = \\{ version = \"[^\"]+\"[^}]*\\}#carbon-core = { path = \"$core_path\", features = [\"macros\"], default-features = false }#g" "$manifest"
    perl -0pi -e "s#carbon-test-utils = \\{ version = \"[^\"]+\" \\}#carbon-test-utils = { path = \"$test_utils_path\" }#g" "$manifest"
}

run_regeneration_checks() {
    local entries_file="$1"
    local count=0

    if [[ ! -s "$entries_file" ]]; then
        echo "No regeneration entries discovered." >&2
        return 1
    fi

    pnpm --filter @sevenlabs-hq/carbon-cli build

    temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/carbon-clickhouse-rollout.XXXXXX")"

    while IFS='|' read -r name source; do
        if [[ -z "$name" || -z "$source" ]]; then
            continue
        fi
        if [[ "$REGENERATE_LIMIT" -gt 0 && "$count" -ge "$REGENERATE_LIMIT" ]]; then
            break
        fi

        local output_dir="${temp_dir}/${name}"
        local args=(
            packages/cli/dist/cli.js
            parse
            --idl "$source"
            --out-dir "$output_dir"
            --standard "$REGENERATE_STANDARD"
            --name "$name"
            --with-postgres false
            --with-graphql false
            --with-clickhouse true
            --with-serde true
            --standalone true
        )
        if [[ "$source" != *.json ]]; then
            args+=(--url "$REGENERATE_RPC_URL")
        fi

        "$NODE_BIN" "${args[@]}"

        if ! grep -q '^clickhouse = \[' "$output_dir/Cargo.toml"; then
            echo "Regenerated $name is missing clickhouse feature" >&2
            return 1
        fi
        if ! find "$output_dir/src" -type d -name clickhouse -print -quit | grep -q .; then
            echo "Regenerated $name is missing generated clickhouse modules" >&2
            return 1
        fi

        if [[ "$SKIP_REGENERATED_COMPILE" -eq 0 ]]; then
            patch_generated_manifest_for_local_core "$output_dir/Cargo.toml"
            cp "$ROOT_DIR/Cargo.lock" "$output_dir/Cargo.lock"
            cargo check --manifest-path "$output_dir/Cargo.toml" --features clickhouse --offline
        fi

        count=$((count + 1))
    done < "$entries_file"

    printf 'Regenerated %d ClickHouse-enabled decoder crates in a temporary directory.\n' "$count"
}

if [[ "$SKIP_RENDERER" -eq 0 ]]; then
    pnpm --filter @sevenlabs-hq/carbon-codama-renderer test
    pnpm --filter @sevenlabs-hq/carbon-codama-renderer type-check
fi

mapfile -t manifests < <(find decoders -mindepth 2 -maxdepth 2 -name Cargo.toml | sort)

clickhouse_packages=()
checked_packages=()
violations=()

for manifest in "${manifests[@]}"; do
    decoder_dir="$(dirname "$manifest")"
    package="$(package_name "$manifest")"
    if [[ -z "$package" ]]; then
        violations+=("$manifest has no package name")
        continue
    fi

    feature=0
    module=0
    if has_clickhouse_feature "$manifest"; then
        feature=1
    fi
    if has_clickhouse_module "$decoder_dir"; then
        module=1
    fi

    if [[ "$ALLOW_BROAD_CLICKHOUSE" -eq 0 ]] && ! is_canary_decoder "$package"; then
        if [[ "$feature" -eq 1 ]]; then
            violations+=("$package has a clickhouse feature but is not a committed canary")
        fi
        if [[ "$module" -eq 1 ]]; then
            violations+=("$package has generated clickhouse modules but is not a committed canary")
        fi
    fi

    if [[ "$feature" -eq 1 ]]; then
        clickhouse_packages+=("$package")
    fi

    if [[ "$COMPILE_ALL" -eq 1 ]]; then
        cargo check -p "$package"
        checked_packages+=("$package")
    fi
done

if [[ "${#violations[@]}" -gt 0 ]]; then
    printf 'ClickHouse decoder rollout violations:\n' >&2
    printf '  - %s\n' "${violations[@]}" >&2
    exit 1
fi

for package in "${clickhouse_packages[@]}"; do
    cargo check -p "$package" --features clickhouse
done

if [[ -n "$REGENERATE_IDL_DIR" || "$REGENERATE_FROM_README" -eq 1 ]]; then
    entries_file="$(mktemp "${TMPDIR:-/tmp}/carbon-clickhouse-rollout-entries.XXXXXX")"
    trap 'rm -f "$entries_file"; cleanup' EXIT

    if [[ -n "$REGENERATE_IDL_DIR" ]]; then
        if [[ ! -d "$REGENERATE_IDL_DIR" ]]; then
            echo "IDL directory does not exist: $REGENERATE_IDL_DIR" >&2
            exit 2
        fi
        while IFS= read -r idl; do
            name="$(basename "$idl" .json)"
            printf '%s|%s\n' "$name" "$idl" >> "$entries_file"
        done < <(find "$REGENERATE_IDL_DIR" -maxdepth 1 -type f -name '*.json' | sort)
    fi

    if [[ "$REGENERATE_FROM_README" -eq 1 ]]; then
        while IFS='|' read -r package program_id; do
            printf '%s|%s\n' "$(decoder_name_from_package "$package")" "$program_id" >> "$entries_file"
        done < <(readme_programs)
    fi

    run_regeneration_checks "$entries_file"
fi

if [[ "$COMPILE_ALL" -eq 1 ]]; then
    printf 'Checked %d decoder packages without extra features.\n' "${#checked_packages[@]}"
fi
printf 'Checked %d decoder packages with clickhouse feature.\n' "${#clickhouse_packages[@]}"

after_status="$(git status --porcelain)"
if [[ "$before_status" != "$after_status" ]]; then
    printf 'Working tree changed while running validation.\n' >&2
    printf 'Before:\n%s\n' "$before_status" >&2
    printf 'After:\n%s\n' "$after_status" >&2
    exit 1
fi
