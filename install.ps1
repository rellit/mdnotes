# mdnotes Windows Installer
# Usage (PowerShell): iwr -useb https://raw.githubusercontent.com/rellit/mdnotes/main/install.ps1 | iex
#Requires -Version 5.1
$ErrorActionPreference = "Stop"

$Repo       = "rellit/mdnotes"
$Archive    = "mdnotes-windows-x86_64.zip"
$InstallDir = "$env:LOCALAPPDATA\mdnotes\bin"

Write-Host "Installing mdnotes..."

# Create install directory
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Download latest release archive
$LatestUrl = "https://github.com/$Repo/releases/latest/download/$Archive"
$TmpZip    = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "mdnotes-install.zip")
$TmpDir    = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), "mdnotes-install")

Write-Host "Downloading $Archive..."
Invoke-WebRequest -Uri $LatestUrl -OutFile $TmpZip

# Extract archive
if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
Expand-Archive -Path $TmpZip -DestinationPath $TmpDir -Force
Remove-Item -Force $TmpZip

# Copy binaries to install directory
foreach ($bin in @("mdn.exe", "mdnui.exe")) {
    $src = Join-Path $TmpDir $bin
    if (Test-Path $src) {
        Copy-Item $src -Destination $InstallDir -Force
        Write-Host "Installed $bin -> $InstallDir\$bin"
    }
}
Remove-Item -Recurse -Force $TmpDir

# Add install directory to the current user's PATH (persistent)
$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH",
        "$CurrentPath;$InstallDir",
        "User"
    )
    Write-Host ""
    Write-Host "Added $InstallDir to your user PATH."
    Write-Host "Restart your terminal for the change to take effect."
}

Write-Host ""
Write-Host "mdnotes installed successfully!"
Write-Host "Run 'mdn --help' to get started."
