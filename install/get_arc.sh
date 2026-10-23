#!/bin/sh
# arc installer — Linux / macOS / WSL
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/krakjn/arc/main/install/get_arc.sh | sh
#
# No GitHub token is required. GITHUB_TOKEN / GH_TOKEN are optional (CI rate limits).
#
set -eu

REPO="krakjn/arc"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { printf '%b[INFO]%b %s\n'    "$BLUE"  "$NC" "$*"; }
success() { printf '%b[SUCCESS]%b %s\n' "$GREEN" "$NC" "$*"; }
warn()    { printf '%b[WARN]%b %s\n'    "$YELLOW" "$NC" "$*"; }
die()     { printf '%b[ERROR]%b %s\n'   "$RED"   "$NC" "$*" >&2; exit 1; }

printf '%s\n' \
"   _____ ___________________
  /  _  \\\\______   \\_   ____\\
 /  /_\\  \\|       _/    \\       A_ggregate
/    |    \\    |   \\     \\____  R_epository
\\____|____/____|___/\\________/  C_ontrol"

command -v curl >/dev/null 2>&1 || die "curl is required"

DEST=""

while [ $# -gt 0 ]; do
  case $1 in
    --dest)
      [ $# -ge 2 ] || die "--dest requires a path"
      DEST=$2
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--dest DEST]"
      echo ""
      echo "  --dest PATH   Install arc to PATH/arc"
      echo "                (default: the arc already on PATH, or /usr/local/bin)"
      echo ""
      echo "When piped from curl, pass options after sh -s --:"
      echo "  curl -fsSL ... | sh -s -- --dest \"\$HOME/.local/bin\""
      exit 0
      ;;
    *)
      die "Unknown option: $1 (try --help)"
      ;;
  esac
done

platform=$(uname -m)-$(uname -s | cut -d- -f1)
case "$platform" in
  *-MINGW64_NT|*-MSYS_NT|*-CYGWIN_NT)
    die "Native Windows — use install/get_arc.ps1"
    ;;
  x86_64-Linux)   os=linux; arch=amd64 ;;
  aarch64-Linux)  os=linux; arch=arm64 ;;
  x86_64-Darwin)  os=macos; arch=amd64 ;;
  arm64-Darwin)   os=macos; arch=arm64 ;;
  *) die "Unsupported platform: $platform" ;;
esac
info "Platform: $os/$arch"

if [ -n "$DEST" ]; then
  target=$DEST/arc
elif command -v arc >/dev/null 2>&1; then
  target=$(command -v arc)
  info "Updating $(arc --version 2>/dev/null || echo arc) at $target"
else
  target=/usr/local/bin/arc
  info "Installing to $target"
fi
if [ -n "$DEST" ]; then
  info "Installing to $target"
fi
dir=$(dirname "$target")

TOKEN="${GITHUB_TOKEN:-${GH_TOKEN:-}}"
if [ -n "$TOKEN" ]; then
  json=$(curl -fsSL -H "Authorization: Bearer $TOKEN" \
    -H "Accept: application/vnd.github+json" \
    "https://api.github.com/repos/$REPO/releases/latest") || die "Could not reach https://api.github.com"
else
  json=$(curl -fsSL -H "Accept: application/vnd.github+json" \
    "https://api.github.com/repos/$REPO/releases/latest") || die "Could not reach https://api.github.com"
fi

tag=$(printf '%s\n' "$json" \
  | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)

[ -n "$tag" ] || die "Could not resolve latest release"
info "Latest release: $tag"

# Public release asset — never send auth (invalid tokens cause 401)
ver=${tag#v}
asset_url="https://github.com/$REPO/releases/download/$tag/arc-${ver}-${os}-${arch}"
info "Downloading: $asset_url"

tmp=$(mktemp) || die "Could not create temporary file"
trap 'rm -f "$tmp"' EXIT INT TERM
curl -fsSL -o "$tmp" "$asset_url" || die "Download failed"
[ -s "$tmp" ] || die "Downloaded file is empty"
chmod +x "$tmp"

if [ -w "$dir" ]; then
  mkdir -p "$dir" || die "Could not create $dir"
  mv "$tmp" "$target" || die "Could not install to $target"
elif command -v sudo >/dev/null 2>&1; then
  info "Elevating with sudo to write $dir"
  sudo mkdir -p "$dir" || die "Could not create $dir"
  sudo mv "$tmp" "$target" || die "Install cancelled or failed"
else
  die "Cannot write to $dir — run with sudo or as root"
fi
trap - EXIT INT TERM

success "Installed arc $("$target" --version 2>/dev/null || echo "$tag") to $target"
