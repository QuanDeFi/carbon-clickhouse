#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

log() { printf '[demo-install] %s\n' "$*"; }
warn() { printf '[demo-install:warn] %s\n' "$*" >&2; }

SUDO=()
if [[ "$(id -u)" -ne 0 ]]; then
  if command -v sudo >/dev/null 2>&1; then
    SUDO=(sudo)
  else
    echo "sudo is required for system package installation" >&2
    exit 1
  fi
fi

export DEBIAN_FRONTEND=noninteractive

log "Installing required system packages"
"${SUDO[@]}" apt-get update
"${SUDO[@]}" apt-get install -y ffmpeg xterm wmctrl xdotool python3-venv

if command -v obs >/dev/null 2>&1; then
  log "OBS already installed: $(obs --version 2>/dev/null || true)"
else
  log "Attempting OBS Studio installation"
  if apt-cache show obs-studio >/dev/null 2>&1; then
    if ! "${SUDO[@]}" apt-get install -y obs-studio; then
      warn "OBS install through apt failed; FFmpeg x11 recording can still be used"
    fi
  else
    warn "obs-studio not found in apt metadata; trying OBS PPA if add-apt-repository exists"
    if command -v add-apt-repository >/dev/null 2>&1; then
      if "${SUDO[@]}" add-apt-repository -y ppa:obsproject/obs-studio \
        && "${SUDO[@]}" apt-get update \
        && "${SUDO[@]}" apt-get install -y obs-studio; then
        log "OBS installed through OBS PPA"
      else
        warn "OBS PPA install failed; FFmpeg x11 recording can still be used"
      fi
    else
      warn "add-apt-repository is unavailable; skipping OBS install"
    fi
  fi
fi

log "Creating Python venv at .venv-demo"
python3 -m venv .venv-demo
.venv-demo/bin/python -m pip install --upgrade pip
.venv-demo/bin/pip install pyyaml requests python-dotenv obsws-python websocket-client rich 'dashscope>=1.25.2'

log "Installing demo-local Playwright dependencies"
npm --prefix scripts/demo/playwright install

log "Install step complete"
