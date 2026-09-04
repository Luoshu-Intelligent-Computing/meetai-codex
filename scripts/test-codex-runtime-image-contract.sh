#!/usr/bin/env bash
set -euo pipefail
ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
DOCKERFILE="$ROOT/packaging/codex-runtime/Dockerfile"
SCRIPT="$ROOT/scripts/build-codex-runtime-image.sh"
test -f "$DOCKERFILE"
test -x "$SCRIPT"
bash -n "$SCRIPT"
rg -q -- 'codex-app-server' "$DOCKERFILE"
rg -q -- 'CODEX_APP_SERVER_SHA256' "$DOCKERFILE"
rg -q -- 'bubblewrap' "$DOCKERFILE"
rg -q -- 'strip --strip-all' "$SCRIPT"
if rg -q -- 'OPENAI_API_KEY|MEETAI_LLM_API_KEY|DATABASE_URL' "$DOCKERFILE"; then
  printf 'runtime image must not contain business/provider secrets\n' >&2
  exit 1
fi
printf 'codex runtime image contract: passed\n'
