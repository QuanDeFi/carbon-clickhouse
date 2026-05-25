#!/usr/bin/env bash
set -euo pipefail

repo_root() {
  git rev-parse --show-toplevel 2>/dev/null || {
    cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd
  }
}

log_info() {
  printf '[demo] %s\n' "$*"
}

log_warn() {
  printf '[demo:warn] %s\n' "$*" >&2
}

log_error() {
  printf '[demo:error] %s\n' "$*" >&2
}

load_demo_env() {
  local root
  root="$(repo_root)"
  local initial_keys
  initial_keys="$(env | cut -d= -f1)"

  key_was_initial() {
    grep -qx "$1" <<< "$initial_keys"
  }

  load_env_file() {
    local file="$1"
    local override_loaded="$2"
    [[ -f "$file" ]] || return 0
    while IFS= read -r line || [[ -n "$line" ]]; do
      line="${line#"${line%%[![:space:]]*}"}"
      line="${line%"${line##*[![:space:]]}"}"
      [[ -z "$line" || "$line" == \#* || "$line" != *=* ]] && continue
      local key="${line%%=*}"
      local value="${line#*=}"
      key="${key%"${key##*[![:space:]]}"}"
      key="${key#"${key%%[![:space:]]*}"}"
      [[ "$key" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || continue
      if [[ -z "${!key+x}" ]]; then
        export "$key=$value"
      elif [[ "$override_loaded" == true ]] && ! key_was_initial "$key"; then
        export "$key=$value"
      fi
    done < "$file"
  }

  local env_file=""
  if [[ -f "$root/.env.demo.local" ]]; then
    env_file="$root/.env.demo.local"
  elif [[ -f "$root/scripts/demo/.env.demo.local" ]]; then
    env_file="$root/scripts/demo/.env.demo.local"
  fi

  if [[ -n "$env_file" ]]; then
    load_env_file "$env_file" false
  fi
  load_env_file "$root/demo-artifacts/display.env" true
}

require_env() {
  local name="$1"
  local value="${!name:-}"
  if [[ -z "$value" || "$value" == \<*\> ]]; then
    log_error "$name must be set in .env.demo.local"
    return 1
  fi
}

mask_secrets() {
  local root
  root="$(repo_root)"
  python3 "$root/scripts/demo/mask_output.py"
}

port_listener() {
  local port="$1"
  ss -ltnp 2>/dev/null | awk -v suffix=":$port" '
    substr($4, length($4) - length(suffix) + 1) == suffix { found=1; print }
    END { exit found ? 0 : 1 }
  '
}

ensure_demo_dirs() {
  local root
  root="$(repo_root)"
  mkdir -p \
    "$root/demo-artifacts/logs" \
    "$root/demo-artifacts/pids" \
    "$root/demo-artifacts/recordings" \
    "$root/demo-artifacts/audio" \
    "$root/demo-artifacts/final-scenes" \
    "$root/demo-artifacts/screenshots" \
    "$root/demo-artifacts/reports" \
    "$root/demo-artifacts/tmp"
}
