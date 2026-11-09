param([switch]$BootstrapSdk)

$ErrorActionPreference = 'Stop'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (Test-Path -LiteralPath $cargoBin) { $env:PATH = "$cargoBin;$env:PATH" }
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
if (!(Test-Path -LiteralPath $vswhere)) { throw 'Installa Microsoft C++ Build Tools (Sviluppo desktop con C++).' }
$visualStudioPath = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (!$visualStudioPath) { throw 'Componente MSVC x64 non trovato. Installalo dai Build Tools.' }
$msvc = Get-ChildItem -LiteralPath (Join-Path $visualStudioPath 'VC\Tools\MSVC') -Directory | Sort-Object Name -Descending | Select-Object -First 1
$msvcBin = Join-Path $msvc.FullName 'bin\Hostx64\x64'
$msvcLib = Join-Path $msvc.FullName 'lib\x64'
$msvcInclude = Join-Path $msvc.FullName 'include'

# The SDK may be installed outside Program Files: the registry holds the real root.
$installedSdk = (Get-ItemProperty -LiteralPath 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows Kits\Installed Roots' -Name KitsRoot10 -ErrorAction SilentlyContinue).KitsRoot10
if (!$installedSdk) { $installedSdk = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10' }
$sdkVersion = Get-ChildItem -LiteralPath (Join-Path $installedSdk 'Lib') -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Where-Object {
    Test-Path -LiteralPath (Join-Path $_.FullName 'um\x64\kernel32.lib')
} | Select-Object -First 1
if ($sdkVersion) {
    $sdkIncludeRoot = Join-Path $installedSdk "Include\$($sdkVersion.Name)"
    $sdkLibRoot = $sdkVersion.FullName
    $sdkBin = Join-Path $installedSdk "bin\$($sdkVersion.Name)\x64"
} else {
    # Official Microsoft SDK packages; local cache, no administrative install.
    $sdkPackageVersion = '10.0.28000.2705'
    $sdkCache = Join-Path $env:LOCALAPPDATA "MonitorAwake\build-tools\$sdkPackageVersion"
    foreach ($package in @('microsoft.windows.sdk.cpp', 'microsoft.windows.sdk.cpp.x64')) {
        $destination = Join-Path $sdkCache $package
        if (!(Test-Path -LiteralPath (Join-Path $destination '.complete'))) {
            if (!$BootstrapSdk) { throw 'Windows SDK mancante. Esegui: . .\scripts\windows-env.ps1 -BootstrapSdk' }
            New-Item -ItemType Directory -Path $destination -Force | Out-Null
            $archive = Join-Path $sdkCache "$package.zip"
            Write-Host "Scarico $package $sdkPackageVersion dal feed ufficiale NuGet..."
            Invoke-WebRequest -UseBasicParsing -Uri "https://api.nuget.org/v3-flatcontainer/$package/$sdkPackageVersion/$package.$sdkPackageVersion.nupkg" -OutFile $archive
            Expand-Archive -LiteralPath $archive -DestinationPath $destination -Force
            Set-Content -LiteralPath (Join-Path $destination '.complete') -Value $sdkPackageVersion
        }
    }
    $commonRoot = Join-Path $sdkCache 'microsoft.windows.sdk.cpp'
    $archRoot = Join-Path $sdkCache 'microsoft.windows.sdk.cpp.x64'
    $includeVersion = Get-ChildItem -LiteralPath (Join-Path $commonRoot 'c\Include') -Directory | Sort-Object Name -Descending | Select-Object -First 1
    $binVersion = Get-ChildItem -LiteralPath (Join-Path $commonRoot 'c\bin') -Directory | Where-Object { $_.Name -match '^10\.' } | Sort-Object Name -Descending | Select-Object -First 1
    $sdkIncludeRoot = $includeVersion.FullName
    $sdkLibRoot = Join-Path $archRoot 'c'
    $sdkBin = Join-Path $binVersion.FullName 'x64'
}
$env:PATH = "$msvcBin;$sdkBin;$env:PATH"
$env:LIB = "$msvcLib;$sdkLibRoot\um\x64;$sdkLibRoot\ucrt\x64"
$env:INCLUDE = "$msvcInclude;$sdkIncludeRoot\ucrt;$sdkIncludeRoot\shared;$sdkIncludeRoot\um;$sdkIncludeRoot\winrt"
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER = Join-Path $msvcBin 'link.exe'
$env:RC = Join-Path $sdkBin 'rc.exe'
if (!(Test-Path -LiteralPath $env:RC) -or !(Test-Path -LiteralPath "$sdkLibRoot\um\x64\kernel32.lib")) { throw 'Windows SDK incompleto: mancano rc.exe o kernel32.lib.' }
Write-Host 'Ambiente Rust/MSVC x64 pronto per questa sessione PowerShell.'
