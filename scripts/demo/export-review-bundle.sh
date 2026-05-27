#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

CREATE_ARCHIVE=true
if [[ "${1:-}" == "--no-archive" ]]; then
  CREATE_ARCHIVE=false
  shift
fi

REVIEW_DIR="${1:-demo-artifacts/review-human}"
REVIEW_TS="$(cat "$REVIEW_DIR/latest-review-timestamp.txt" 2>/dev/null || date +%Y%m%d-%H%M%S)"
ARCHIVE_DIR="$REVIEW_DIR/archive"

automation="$REVIEW_DIR/automation-scripts.md"
{
  echo '# Demo Automation Scripts'
  echo
  for file in AGENTS.md scripts/demo/README.md scripts/demo/runbook.yaml scripts/demo/*.sh scripts/demo/*.py scripts/demo/*.js scripts/demo/playwright/package.json scripts/demo/playwright/playwright.config.ts scripts/demo/playwright/*.spec.ts; do
    [[ -f "$file" ]] || continue
    echo "## \`$file\`"
    echo
    case "$file" in
      *.py) echo '```python' ;;
      *.sh) echo '```bash' ;;
      *.js) echo '```js' ;;
      *.ts) echo '```ts' ;;
      *.json) echo '```json' ;;
      *.yaml|*.yml) echo '```yaml' ;;
      *.md) echo '```md' ;;
      *) echo '```text' ;;
    esac
    python3 - "$file" <<'PY'
from pathlib import Path
import re
import sys

text = Path(sys.argv[1]).read_text(errors="ignore")
text = re.sub(r"sk_[A-Za-z0-9]+", "<redacted-api-key>", text)
text = re.sub(
    r"https?://[^\s\"'<>]*(?:api-key|apikey|token|alchemy|helius|quicknode|syndica|chainstack|drpc|getblock|shyft)[^\s\"'<>]*",
    "<redacted-url>",
    text,
    flags=re.IGNORECASE,
)
sys.stdout.write(text)
PY
    echo '```'
    echo
  done
} > "$automation"

if [[ "$CREATE_ARCHIVE" == "true" ]]; then
  mkdir -p "$ARCHIVE_DIR"
  archive="$ARCHIVE_DIR/clickhouse-tutorial-human-review-bundle-${REVIEW_TS}.tar.gz"
  tar -czf "$archive" \
    "$REVIEW_DIR/manifest.md" \
    "$REVIEW_DIR/scene-scripts.md" \
    "$REVIEW_DIR/runbook.yaml" \
    "$REVIEW_DIR/automation-scripts.md" \
    "$REVIEW_DIR/frame-contact-sheet.jpg" \
    "$REVIEW_DIR/videos" \
    "$REVIEW_DIR/subtitles" \
    "$REVIEW_DIR/screenshots" \
    "$REVIEW_DIR/logs" \
    scripts/demo/README.md \
    scripts/demo/runbook.yaml \
    docs/tutorial-video \
    2>/dev/null
  echo "$archive"
else
  echo "archive skipped"
fi
