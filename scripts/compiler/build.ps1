$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$OutDir = Join-Path $RootDir "dist\compiler\windows"
$Target = if ($env:PEPS_WINDOWS_TARGET) { $env:PEPS_WINDOWS_TARGET } else { "x86_64-pc-windows-msvc" }
$TargetReleaseDir = Join-Path $RootDir "target\$Target\release"

Set-Location $RootDir

cargo build --release --bin peps --target $Target
Remove-Item $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Copy-Item (Join-Path $TargetReleaseDir "peps.exe") (Join-Path $OutDir "peps!.exe") -Force

@"
@echo off
set DIR=%~dp0
"%DIR%peps!.exe" %*
"@ | Set-Content -Encoding ASCII (Join-Path $OutDir "peps.cmd")

Write-Host "Built Peps compiler Windows dist: dist\compiler\windows"
Write-Host "Windows target: $Target"
Write-Host "Manual run: .\dist\compiler\windows\peps!.exe path\to\file.peps"
