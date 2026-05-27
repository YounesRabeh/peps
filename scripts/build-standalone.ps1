$ErrorActionPreference = "Stop"

cargo build --release
New-Item -ItemType Directory -Force -Path dist | Out-Null
Copy-Item target\release\peps.exe dist\peps.exe -Force

Write-Host "Built standalone runner: dist\peps.exe"
Write-Host "Run it with: .\dist\peps.exe path\to\file.peps"
