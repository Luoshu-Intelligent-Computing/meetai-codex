#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
Usage: build-codex-runtime-image.sh --source FILE [--tag IMAGE] [--runtime podman|docker]

Build the minimal Codex app-server runtime image from a pinned executable.
The source is copied to a temporary context and stripped; no binary is added
to the git worktree.
EOF
}
die() { printf '[codex-runtime-image] ERROR: %s\n' "$*" >&2; exit 2; }

source_file=''; image=''; runtime="${CONTAINER_RUNTIME:-podman}"
while (($#)); do
  case "$1" in
    --source) [[ $# -ge 2 ]] || die "--source requires a value"; source_file=$2; shift 2 ;;
    --tag) [[ $# -ge 2 ]] || die "--tag requires a value"; image=$2; shift 2 ;;
    --runtime) [[ $# -ge 2 ]] || die "--runtime requires a value"; runtime=$2; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done

[[ -n "$source_file" ]] || die "--source is required"
source_file=$(realpath -e "$source_file") || die "source does not exist"
[[ -x "$source_file" ]] || die "source is not executable"
command -v file >/dev/null 2>&1 || die "file is required"
command -v strip >/dev/null 2>&1 || die "strip is required"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required"

format=$(file -Lb "$source_file")
[[ "$format" == *'ELF '* ]] || die "source is not an ELF binary: $format"
case "$(uname -m)" in
  x86_64|amd64) [[ "$format" == *'x86-64'* ]] || die "source architecture mismatch: $format" ;;
  aarch64|arm64) [[ "$format" == *'ARM aarch64'* ]] || die "source architecture mismatch: $format" ;;
esac

case "$runtime" in
  podman|docker) ;;
  *) die "runtime must be podman or docker" ;;
esac
command -v "$runtime" >/dev/null 2>&1 || die "$runtime is required"

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
context=$(mktemp -d /tmp/meetai-codex-runtime.XXXXXX)
cp -- "$source_file" "$context/codex-app-server"
strip --strip-all -- "$context/codex-app-server"
digest=$(sha256sum "$context/codex-app-server" | awk '{print $1}')
size=$(stat -c '%s' "$context/codex-app-server")
case "$(uname -m)" in
  x86_64|amd64) target_arch=amd64 ;;
  aarch64|arm64) target_arch=arm64 ;;
  *) die "unsupported target architecture: $(uname -m)" ;;
esac
commit="${MEETAI_CODEX_COMMIT:-$(git -C "$repo_root" rev-parse HEAD 2>/dev/null || printf unknown)}"
image="${image:-registry.cn-hangzhou.aliyuncs.com/meetai/codex-runtime:${commit}-$(uname -m)}"

"$runtime" build \
  -f "$repo_root/packaging/codex-runtime/Dockerfile" \
  -t "$image" \
  --build-arg "CODEX_COMMIT=$commit" \
  --build-arg "CODEX_APP_SERVER_SHA256=$digest" \
  --build-arg "CODEX_APP_SERVER_SIZE=$size" \
  --build-arg "TARGETARCH=$target_arch" \
  "$context"

printf 'image=%s\ncodex_commit=%s\nsha256=%s\nsize_bytes=%s\narch=%s\ncontext=%s\n' \
  "$image" "$commit" "$digest" "$size" "$target_arch" "$context"
