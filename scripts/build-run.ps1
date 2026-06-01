$ErrorActionPreference = "Stop"

$RootDir = Resolve-Path (Join-Path $PSScriptRoot "..")
$Mode = if ($args.Count -gt 0) { $args[0] } else { "all" }
$Rest = if ($args.Count -gt 1) { $args[1..($args.Count - 1)] } else { @() }

if (-not $env:PEPS_WINDOWS_TARGET) {
    $env:PEPS_WINDOWS_TARGET = "x86_64-pc-windows-msvc"
}

switch ($Mode) {
    "compiler" {
        $SourceFile = if ($Rest.Count -gt 0) { $Rest[0] } else { "examples\basic.peps" }
        & (Join-Path $RootDir "scripts\compiler\build.ps1")
        & (Join-Path $RootDir "dist\compiler\peps!.exe") $SourceFile
    }
    "ide" {
        & (Join-Path $RootDir "scripts\ide\build.ps1")
        & (Join-Path $RootDir "dist\ide\peps-ide.exe") @Rest
    }
    "all" {
        $SourceFile = if ($Rest.Count -gt 0) { $Rest[0] } else { "examples\basic.peps" }
        $IdeArgs = if ($Rest.Count -gt 1) { $Rest[1..($Rest.Count - 1)] } else { @() }

        & (Join-Path $RootDir "scripts\compiler\build.ps1")
        & (Join-Path $RootDir "dist\compiler\peps!.exe") $SourceFile

        & (Join-Path $RootDir "scripts\ide\build.ps1")
        & (Join-Path $RootDir "dist\ide\peps-ide.exe") @IdeArgs
    }
    default {
        Write-Error "Usage: .\scripts\build-run.ps1 [compiler [file.peps] | ide | all [file.peps]]"
    }
}
