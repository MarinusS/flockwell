#!/usr/bin/env bash
set -euo pipefail

# Resolve .env relative to this script, regardless of the caller's directory.
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"

if [[ ! -f .env ]]; then
    echo "Missing .env. Copy .env.example to .env and set your credentials path." >&2
    exit 1
fi

# Export assignments from our trusted local Bash configuration.
set -a
source .env
set +a

: "${FLOCKWELL_GOOGLE_CREDENTIALS_JSON:?Set FLOCKWELL_GOOGLE_CREDENTIALS_JSON in .env}"

exec cargo run -p flockwell-audit "$@"
