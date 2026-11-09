param([switch]$BootstrapSdk)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot

Push-Location -LiteralPath $projectRoot
try {
    & (Join-Path $PSScriptRoot 'windows-env.ps1') -BootstrapSdk:$BootstrapSdk

    if (!(Get-Command npm.cmd -ErrorAction SilentlyContinue)) {
        throw 'Node.js non trovato. Installa Node.js e riapri PowerShell.'
    }

    if (!(Test-Path -LiteralPath (Join-Path $projectRoot 'node_modules'))) {
        Write-Host 'Installazione delle dipendenze frontend...'
        & npm.cmd ci
        if ($LASTEXITCODE -ne 0) {
            throw "Installazione delle dipendenze fallita (codice $LASTEXITCODE)."
        }
    }

    Write-Host 'Compilazione di MonitorAwake e generazione installer Windows...'
    & npm.cmd run desktop:build -- --bundles nsis
    if ($LASTEXITCODE -ne 0) {
        throw "Compilazione fallita (codice $LASTEXITCODE)."
    }

    Write-Host 'Compilazione completata.'
    Write-Host "Eseguibile: $(Join-Path $projectRoot 'target\release\monitorawake.exe')"
    Write-Host "Installer:  $(Join-Path $projectRoot 'target\release\bundle\nsis')"
} finally {
    Pop-Location
}
