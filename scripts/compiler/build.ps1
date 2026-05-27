$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$OutDir = Join-Path $RootDir "dist\compiler"

Set-Location $RootDir

cargo build --release --bin peps
Remove-Item $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

Copy-Item "target\release\peps.exe" (Join-Path $OutDir "peps!.exe") -Force

@"
@echo off
set DIR=%~dp0
"%DIR%peps!.exe" %*
"@ | Set-Content -Encoding ASCII (Join-Path $OutDir "peps.cmd")

Write-Host "Built Peps compiler runner: dist\compiler"
Write-Host "Run it with: .\dist\compiler\peps!.exe path\to\file.peps"
