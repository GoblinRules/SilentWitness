#Requires -Version 5.1
<#
.SYNOPSIS
    Build a release silentwitness.exe and stage it with LICENSE + README.
.DESCRIPTION
    Intended for local release packaging. Produces rust/dist/silentwitness.exe
    plus supporting files, ready to ship as a portable zip.
#>

[CmdletBinding()]
param(
    [switch]$Clean,
    [switch]$Zip
)

$ErrorActionPreference = 'Stop'
Set-Location -Path $PSScriptRoot

if ($Clean) {
    Write-Host "cargo clean" -ForegroundColor Cyan
    cargo clean
}

Write-Host "cargo build --release --locked" -ForegroundColor Cyan
cargo build --release --locked
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }

$dist = Join-Path $PSScriptRoot 'dist'
Remove-Item -Recurse -Force $dist -ErrorAction SilentlyContinue | Out-Null
New-Item -ItemType Directory -Path $dist | Out-Null

Copy-Item "$PSScriptRoot\target\release\silentwitness.exe" $dist
Copy-Item "$PSScriptRoot\..\LICENSE.md" $dist
Copy-Item "$PSScriptRoot\..\README.md" $dist
Copy-Item "$PSScriptRoot\README.md" (Join-Path $dist 'README-RUST.md')

Write-Host "Staged to: $dist" -ForegroundColor Green
Get-ChildItem $dist | Format-Table Name, Length

if ($Zip) {
    $zipPath = Join-Path $PSScriptRoot 'silentwitness-windows-x86_64.zip'
    Remove-Item -Force $zipPath -ErrorAction SilentlyContinue
    Compress-Archive -Path "$dist\*" -DestinationPath $zipPath
    Write-Host "Zip: $zipPath" -ForegroundColor Green
}
