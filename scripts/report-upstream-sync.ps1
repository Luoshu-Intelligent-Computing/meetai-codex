[CmdletBinding()]
param(
  [string]$UpstreamRef = 'upstream/main',
  [string]$BaseRef = 'HEAD',
  [switch]$Fetch
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Invoke-GitLines {
  param([string[]]$Arguments)

  $lines = & git @Arguments
  if ($LASTEXITCODE -ne 0) {
    throw "git $($Arguments -join ' ') failed with exit code $LASTEXITCODE"
  }
  return @($lines)
}

function Get-CommitLines {
  param(
    [string]$Range,
    [string[]]$Paths,
    [int]$Limit = 12
  )

  $arguments = @('log', '--format=%h%x09%s', "-$Limit", $Range, '--') + $Paths
  $lines = Invoke-GitLines -Arguments $arguments
  return @($lines)
}

if ($Fetch) {
  Invoke-GitLines -Arguments @(
    'fetch', 'upstream', 'main', '--no-tags', '--filter=blob:none'
  ) | Out-Null
}

Invoke-GitLines -Arguments @('rev-parse', '--verify', $BaseRef) | Out-Null
Invoke-GitLines -Arguments @('rev-parse', '--verify', $UpstreamRef) | Out-Null

$mergeBase = @(Invoke-GitLines -Arguments @('merge-base', $BaseRef, $UpstreamRef))[0]
$upstreamHead = @(Invoke-GitLines -Arguments @('rev-parse', $UpstreamRef))[0]
$localHead = @(Invoke-GitLines -Arguments @('rev-parse', $BaseRef))[0]
$aheadBehindLine = @(
  Invoke-GitLines -Arguments @(
    'rev-list', '--left-right', '--count', "$BaseRef...$UpstreamRef"
  )
)[0]
$aheadBehind = @($aheadBehindLine -split '\s+' | Where-Object { $_ })
if ($aheadBehind.Count -ne 2) {
  throw "Expected two commit counts, received: $aheadBehindLine"
}
$range = "$mergeBase..$UpstreamRef"

$categories = @(
  @{
    Name = 'P0 app-server and protocol'
    Paths = @('codex-rs/app-server', 'codex-rs/app-server-protocol', 'codex-rs/protocol')
    Action = 'Require fixture review and Desktop adapter contract verification.'
  },
  @{
    Name = 'P1 skills and MCP'
    Paths = @('codex-rs/skills', 'codex-rs/codex-mcp', 'codex-rs/mcp-types', 'codex-rs/core/src/skills')
    Action = 'Review packaged skills, MCP lifecycle, auth, and approval implications.'
  },
  @{
    Name = 'P1 provider and model runtime'
    Paths = @('codex-rs/core/src/client', 'codex-rs/core/src/model_provider_info', 'codex-rs/protocol/src/models', 'codex-rs/responses-api-proxy')
    Action = 'Review provider compatibility before updating MeetAI gateway configuration.'
  },
  @{
    Name = 'P3 CLI and TUI only'
    Paths = @('codex-rs/cli', 'codex-rs/tui', 'codex-cli')
    Action = 'Normally exclude from Desktop runtime integration unless a shared protocol dependency requires it.'
  }
)

$report = [System.Collections.Generic.List[string]]::new()
$report.Add('# Codex Upstream Sync Report')
$report.Add('')
$report.Add("Generated (UTC): $([DateTime]::UtcNow.ToString('yyyy-MM-dd HH:mm:ss'))")
$report.Add("Base: $BaseRef ($localHead)")
$report.Add("Upstream: $UpstreamRef ($upstreamHead)")
$report.Add("Merge base: $mergeBase")
$report.Add("Fork-only commits: $($aheadBehind[0])")
$report.Add("Upstream-only commits: $($aheadBehind[1])")
$report.Add('')
$report.Add('This report is review input only. It does not merge, rebase, reset, or create branches.')

foreach ($category in $categories) {
  $commits = @(Get-CommitLines -Range $range -Paths $category.Paths)
  $report.Add('')
  $report.Add("## $($category.Name)")
  $report.Add($category.Action)
  $report.Add("Matched commits: $($commits.Count)")
  if ($commits.Count -eq 0) {
    $report.Add('- None in the current range.')
    continue
  }
  foreach ($commit in $commits) {
    $parts = $commit -split "`t", 2
    $summary = if ($parts.Count -gt 1) { $parts[1] } else { '' }
    $report.Add("- $($parts[0]) $summary")
  }
}

$report.Add('')
$report.Add('## Integration Gate')
$report.Add('- Create a dedicated integration branch only after P0/P1 changes are reviewed.')
$report.Add('- Before promotion, verify app-server protocol fixtures, MCP/skills, approval, recovery, and Windows package consumption.')
$report.Add('- Do not cherry-pick individual commits from a large upstream range without a dependency review.')

$report -join [Environment]::NewLine
