#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <scene-id> <command> [args...]" >&2
  exit 2
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

SCENE_ID="$1"
shift
DISPLAY="${DISPLAY:-${DEMO_DISPLAY:-:99}}"
export DISPLAY

command -v xterm >/dev/null 2>&1 || {
  log_error "xterm is required for terminal scenes"
  exit 1
}

quoted=""
for arg in "$@"; do
  printf -v q '%q' "$arg"
  quoted+="${quoted:+ }$q"
done

SCRIPT="$ROOT/demo-artifacts/tmp/$SCENE_ID-terminal.sh"
STATUS_FILE="$ROOT/demo-artifacts/tmp/$SCENE_ID-terminal.status"
cat > "$SCRIPT" <<EOF
#!/usr/bin/env bash
set -o pipefail
cd "$ROOT"
cmd=($quoted)
clear
echo "Carbon ClickHouse Tutorial - $SCENE_ID"
echo
echo "\$ $quoted"
echo
"\${cmd[@]}" 2>&1 | "$ROOT/scripts/demo/mask_output.py"
status=\${PIPESTATUS[0]}
echo
echo "Command exited with status \$status"
sleep "\${DEMO_TERMINAL_HOLD_SECONDS:-5}"
echo "\$status" > "$STATUS_FILE"
exit "\$status"
EOF
chmod +x "$SCRIPT"
rm -f "$STATUS_FILE"

xterm -T "$SCENE_ID" -geometry 160x45 -fa Monospace -fs 12 -e "$SCRIPT"
status="$(cat "$STATUS_FILE" 2>/dev/null || echo 1)"
exit "$status"
