$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$OutDir = Join-Path $RootDir "dist\ide\windows"
$Target = if ($env:PEPS_WINDOWS_TARGET) { $env:PEPS_WINDOWS_TARGET } else { "x86_64-pc-windows-msvc" }
$TargetReleaseDir = Join-Path $RootDir "target\$Target\release"

Set-Location $RootDir

$Version = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
if (-not $Version) {
    throw "Could not read package version from Cargo.toml"
}
$IdeName = "peps-ide-$Version.exe"
$LauncherName = "peps-ide-$Version.cmd"

if (-not (Test-Path "Cargo.toml")) {
    throw "Cargo.toml not found at project root: $RootDir"
}

if (-not (Test-Path "ide")) {
    throw "ide\ directory not found at project root: $RootDir"
}

Push-Location ide
try {
    pnpm install --frozen-lockfile
    pnpm run build
} finally {
    Pop-Location
}

if (-not (Test-Path "ide\dist\index.html")) {
    throw "Frontend build did not produce ide\dist\index.html"
}

cargo build --release --bin peps-ide --target $Target

Remove-Item $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "frontend") | Out-Null

Copy-Item (Join-Path $TargetReleaseDir "peps-ide.exe") (Join-Path $OutDir $IdeName) -Force
Copy-Item "ide\dist" (Join-Path $OutDir "frontend\dist") -Recurse -Force

@"
@echo off
set DIR=%~dp0
cd /d "%DIR%"
"%DIR%$IdeName" %*
"@ | Set-Content -Encoding ASCII (Join-Path $OutDir $LauncherName)

Write-Host "Built Peps IDE Windows dist: dist\ide\windows"
Write-Host "Version: $Version"
Write-Host "Windows target: $Target"
Write-Host "Manual start: .\dist\ide\windows\$LauncherName"
