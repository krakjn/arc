#!/bin/sh
# arc installer — Linux / macOS / WSL
#
# Usage:
#   curl -fsSL https://gitlab.com/ballast-dev/arc/-/raw/main/install/get_arc.sh | sh
#
# Optional: GITLAB_TOKEN / CI_JOB_TOKEN for private projects / rate limits.
#
set -eu

# GitLab project path (namespace/name)
PROJECT="ballast-dev/arc"
API_BASE="https://gitlab.com/api/v4"
PROJECT_ENC=$(printf '%s' "$PROJECT" | sed 's|/|%2F|g')

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

platform=$(uname -m)-$(uname -s | cut -d- -f1)
case "$platform" in
  *-MINGW64_NT|*-MSYS_NT|*-CYGWIN_NT)
    die "Native Windows is not supported — use install/get_arc.ps1"
    ;;
  x86_64-Linux)   os=linux; arch=amd64 ;;
  aarch64-Linux)  os=linux; arch=arm64 ;;
  x86_64-Darwin)  os=macos; arch=amd64 ;;
  arm64-Darwin)   os=macos; arch=arm64 ;;
  *) die "Unsupported platform: $platform" ;;
esac
info "Platform: $os/$arch"

asset="arc-${os}-${arch}"

# Update in place when arc is already on PATH
if command -v arc >/dev/null 2>&1; then
  target=$(command -v arc)
  version=$(arc --version 2>/dev/null || echo arc)
  info "Updating $version at $target"
else
  target=/usr/local/bin/arc
  info "Installing to $target"
fi
dir=$(dirname "$target")

TOKEN="${GITLAB_TOKEN:-${CI_JOB_TOKEN:-}}"
curl_auth() {
  if [ -n "$TOKEN" ]; then
    curl -fsSL -H "PRIVATE-TOKEN: ${TOKEN}" "$@"
  else
    curl -fsSL "$@"
  fi
}

# Latest release tag via GitLab API
json=$(curl_auth \
  "${API_BASE}/projects/${PROJECT_ENC}/releases/permalink/latest") \
  || die "Could not reach GitLab releases API"

tag=$(printf '%s\n' "$json" \
  | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)

[ -n "$tag" ] || die "Could not resolve latest release"
info "Latest release: $tag"

# Prefer generic package registry (uploaded by CI), fall back to release download link
asset_url="${API_BASE}/projects/${PROJECT_ENC}/packages/generic/arc/${tag}/${asset}"
info "Downloading: $asset_url"

tmp=$(mktemp) || die "Could not create temporary file"
trap 'rm -f "$tmp"' EXIT INT TERM

if ! curl_auth -o "$tmp" "$asset_url"; then
  asset_url="https://gitlab.com/${PROJECT}/-/releases/${tag}/downloads/${asset}"
  info "Retrying: $asset_url"
  curl_auth -o "$tmp" "$asset_url" || die "Download failed"
fi
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

success "Installed arc $(arc --version 2>/dev/null || echo "$tag") to $target"
