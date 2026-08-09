$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$OutDir = Join-Path $RootDir "dist\compiler\windows"
$Target = if ($env:PEPS_WINDOWS_TARGET) { $env:PEPS_WINDOWS_TARGET } else { "x86_64-pc-windows-msvc" }
$TargetReleaseDir = Join-Path $RootDir "target\$Target\release"

Set-Location $RootDir

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
if (-not $Version) {
    throw "Could not read package version from Cargo.toml"
}
$CompilerName = "peps-$Version.exe"
$LauncherName = "peps-$Version.cmd"

cargo build --release --bin peps --target $Target
Remove-Item $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Copy-Item (Join-Path $TargetReleaseDir "peps.exe") (Join-Path $OutDir $CompilerName) -Force

@"
@echo off
set DIR=%~dp0
"%DIR%$CompilerName" %*
"@ | Set-Content -Encoding ASCII (Join-Path $OutDir "peps.cmd")

Move-Item (Join-Path $OutDir "peps.cmd") (Join-Path $OutDir $LauncherName) -Force

Write-Host "Built Peps compiler Windows dist: dist\compiler\windows"
Write-Host "Version: $Version"
Write-Host "Windows target: $Target"
Write-Host "Manual run: .\dist\compiler\windows\$LauncherName path\to\file.peps"
