# arc installer — Windows (PowerShell 5.1+ / 7+)
#
# Usage:
#   irm https://gitlab.com/ballast-dev/arc/-/raw/main/install/get_arc.ps1 | iex
#
param([string]$InstallDir = '')
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$project = 'ballast-dev/arc'
$projectEnc = [uri]::EscapeDataString($project)
$apiBase = 'https://gitlab.com/api/v4'

$headers = @{ 'User-Agent' = 'arc-installer' }
if ($env:GITLAB_TOKEN) { $headers['PRIVATE-TOKEN'] = $env:GITLAB_TOKEN }
elseif ($env:CI_JOB_TOKEN) { $headers['JOB-TOKEN'] = $env:CI_JOB_TOKEN }

function Die($msg) { Write-Host "[ERROR] $msg" -ForegroundColor Red; exit 1 }
function Info($msg) { Write-Host "[INFO] $msg" }

$arch = switch ($env:PROCESSOR_ARCHITECTURE) {
  'AMD64' { 'amd64' }
  'ARM64' { 'arm64' }
  default { $null }
}
if (-not $arch) { Die "Unsupported architecture: $env:PROCESSOR_ARCHITECTURE" }
Info "Platform: Windows ($arch)"

$asset = "arc-windows-$arch.exe"

# Resolve install target — update in place if arc already exists
$existing = Get-Command arc -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
if ($existing -and $existing.Source -and (Test-Path $existing.Source)) {
  $targetPath = [IO.Path]::GetFullPath($existing.Source)
  $InstallDir = Split-Path $targetPath
  Info "Updating existing arc at $targetPath"
} else {
  if (-not $InstallDir) {
    $InstallDir = if ($env:RUNNER_TEMP) {
      Join-Path $env:RUNNER_TEMP 'arc'
    } else {
      Join-Path $env:LOCALAPPDATA 'Programs\arc'
    }
  }
  $InstallDir = [IO.Path]::GetFullPath($InstallDir)
  $targetPath = Join-Path $InstallDir 'arc.exe'
}

# Fetch latest release tag
$release = Invoke-RestMethod "$apiBase/projects/$projectEnc/releases/permalink/latest" -Headers $headers
if (-not $release.tag_name) { Die 'Could not resolve latest release.' }
$tag = $release.tag_name.Trim()
Info "Latest release: $tag"

# Prefer generic package registry (uploaded by CI)
$url = "$apiBase/projects/$projectEnc/packages/generic/arc/$tag/$asset"
Info "Downloading: $url"
$tmp = Join-Path ([IO.Path]::GetTempPath()) "arc-$([Guid]::NewGuid().ToString('N')).exe"
try {
  Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing -Headers $headers
} catch {
  $url = "https://gitlab.com/$project/-/releases/$tag/downloads/$asset"
  Info "Retrying: $url"
  Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing -Headers $headers
}
if (-not (Test-Path $tmp) -or (Get-Item $tmp).Length -eq 0) { Die 'Download failed or empty.' }

# Install
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null }
Move-Item $tmp $targetPath -Force
Write-Host "[SUCCESS] Installed: $targetPath" -ForegroundColor Green

# PATH registration
if ($env:GITHUB_PATH) {
  $InstallDir | Out-File $env:GITHUB_PATH -Encoding utf8 -Append
} elseif ($env:GITLAB_ENV) {
  # no-op for GitLab; user PATH below still applies locally
} else {
  $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
  $escaped = [regex]::Escape($InstallDir.TrimEnd('\'))
  if (-not $userPath -or $userPath -notmatch "(?i)(^|;)$escaped(;|$)") {
    $newPath = if ($userPath) { "$userPath;$InstallDir" } else { $InstallDir }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Info "Added to user PATH: $InstallDir"
  }
}
$env:Path += ";$InstallDir"

# Verify
try { Info (& $targetPath --version 2>&1) } catch { }
