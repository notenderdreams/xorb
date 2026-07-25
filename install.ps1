$ErrorActionPreference = 'Stop'

$Repo = "notenderdreams/xorb"
$Asset = "xorb-windows-x86_64.zip"
$DownloadUrl = "https://github.com/$Repo/releases/latest/download/$Asset"

$InstallDir = "$env:LOCALAPPDATA\xorb\bin"
$ExePath = "$InstallDir\xorb.exe"

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$TempZip = [System.IO.Path]::GetTempFileName() + ".zip"
$TempExtract = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())

Write-Host "Downloading xorb for Windows..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $DownloadUrl -OutFile $TempZip

Write-Host "Extracting archive..." -ForegroundColor Cyan
Expand-Archive -Path $TempZip -DestinationPath $TempExtract -Force

if (Test-Path $ExePath) {
    Remove-Item -Path $ExePath -Force -ErrorAction SilentlyContinue
}
Move-Item -Path "$TempExtract\xorb.exe" -Destination $ExePath -Force

Remove-Item -Path $TempZip -Force -ErrorAction SilentlyContinue
Remove-Item -Path $TempExtract -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "✓ xorb successfully installed/updated to $ExePath" -ForegroundColor Green

$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$InstallDir*") {
    $NewPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewPath, [EnvironmentVariableTarget]::User)
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "Added $InstallDir to User PATH." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Run 'xorb --help' in a new terminal to get started!" -ForegroundColor Cyan
