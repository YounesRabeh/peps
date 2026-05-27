$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$OutDir = Join-Path $RootDir "dist\ide"

Set-Location $RootDir

if (-not (Test-Path "Cargo.toml")) {
    throw "Cargo.toml not found at project root: $RootDir"
}

if (-not (Test-Path "ide")) {
    throw "ide\ directory not found at project root: $RootDir"
}

if (Test-Path "ide\package-lock.json") {
    Push-Location ide
    try {
        npm ci
        npm run build
    } finally {
        Pop-Location
    }
} else {
    Push-Location ide
    try {
        npm install
        npm run build
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path "ide\dist\index.html")) {
    throw "Frontend build did not produce ide\dist\index.html"
}

cargo build --release --bin peps-ide

Remove-Item $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path (Join-Path $OutDir "frontend") | Out-Null

Copy-Item "target\release\peps-ide.exe" (Join-Path $OutDir "peps-ide.exe") -Force
Copy-Item "ide\dist" (Join-Path $OutDir "frontend\dist") -Recurse -Force

@"
@echo off
set DIR=%~dp0
cd /d "%DIR%"
"%DIR%peps-ide.exe" %*
"@ | Set-Content -Encoding ASCII (Join-Path $OutDir "peps-ide.cmd")

Write-Host "Built Peps IDE server and frontend: dist\ide"
Write-Host "Start the IDE with: .\dist\ide\peps-ide.cmd"