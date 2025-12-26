#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

NO_CACHE=0
for arg in "$@"; do
  case "$arg" in
    --no-cache) NO_CACHE=1 ;;
    -h|--help)
      echo "Usage: $0 [--no-cache]" >&2
      exit 0
      ;;
    *)
      echo "[setup] error: unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

log() { echo "[setup] $*"; }
die() { echo "[setup] error: $*" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || die "docker is required"
docker compose version >/dev/null 2>&1 || die "docker compose plugin is required (docker compose ...)"

log "Building container (dev-tiktokenizer)..."
(
  cd "$ROOT_DIR"
  if [[ "$NO_CACHE" == "1" ]]; then
    docker compose build --no-cache dev-tiktokenizer
  else
    docker compose build dev-tiktokenizer
  fi
)

log "Starting container (dev-tiktokenizer)..."
(
  cd "$ROOT_DIR"
  docker compose up -d dev-tiktokenizer
)

# Install remembered missing deps (persisted from add-missing-dep.sh)
MISSING_REQ="$ROOT_DIR/scripts/missing-deps.txt"
if [[ -f "$MISSING_REQ" ]]; then
  log "Installing remembered missing deps from scripts/missing-deps.txt..."
  (
    cd "$ROOT_DIR"
    docker compose run --rm dev-tiktokenizer bash -lc "pip install -r /workspace/scripts/missing-deps.txt"
  )
fi

# Optional: install extra dependencies for custom nodes
EXTRA_REQ="$ROOT_DIR/scripts/extra-requirements.txt"
EXTRA_SH="$ROOT_DIR/scripts/extra-deps.sh"
if [[ -f "$EXTRA_REQ" ]]; then
  log "Installing extra requirements from scripts/extra-requirements.txt..."
  (
    cd "$ROOT_DIR"
    docker compose run --rm dev-tiktokenizer bash -lc "pip install -r /workspace/scripts/extra-requirements.txt"
  )
fi
if [[ -f "$EXTRA_SH" ]]; then
  log "Running extra deps script scripts/extra-deps.sh..."
  (
    cd "$ROOT_DIR"
    docker compose run --rm dev-tiktokenizer bash -lc "chmod +x /workspace/scripts/extra-deps.sh && /workspace/scripts/extra-deps.sh"
  )
fi
