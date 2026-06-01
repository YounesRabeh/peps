$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$Mode = if ($args.Count -gt 0) { $args[0] } else { "all" }
$Rest = if ($args.Count -gt 1) { $args[1..($args.Count - 1)] } else { @() }

if (-not $env:PEPS_WINDOWS_TARGET) {
    $env:PEPS_WINDOWS_TARGET = "x86_64-pc-windows-msvc"
}

switch ($Mode) {
    "compiler" {
        & (Join-Path $RootDir "scripts\compiler\build.ps1")
    }
    "ide" {
        & (Join-Path $RootDir "scripts\ide\build.ps1")
    }
    "all" {
        & (Join-Path $RootDir "scripts\compiler\build.ps1")
        & (Join-Path $RootDir "scripts\ide\build.ps1")
    }
    default {
        Write-Error "Usage: .\scripts\build-run.ps1 [compiler | ide | all]"
    }
}
