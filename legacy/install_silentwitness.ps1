# SilentWitness One-Liner Installer
# Usage: irm https://raw.githubusercontent.com/yourusername/silentwitness/main/install_silentwitness.ps1 | iex

Write-Host "SilentWitness One-Click Installer" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "This installer will download and set up SilentWitness completely:" -ForegroundColor White
Write-Host "  - Download the full SilentWitness application" -ForegroundColor Gray
Write-Host "  - Download and install FFmpeg for video recording" -ForegroundColor Gray
Write-Host "  - Download and install Python 3.12 with all dependencies" -ForegroundColor Gray
Write-Host "  - Configure everything automatically" -ForegroundColor Gray
Write-Host ""

# Download and execute the main setup script
$setupScriptUrl = "https://raw.githubusercontent.com/GoblinRules/silentwitness/main/setup_silentwitness.ps1"

Write-Host "Downloading SilentWitness setup script..." -ForegroundColor Yellow
Write-Host "   URL: $setupScriptUrl" -ForegroundColor Gray

try {
    # Download the setup script content
    $setupScript = Invoke-RestMethod -Uri $setupScriptUrl -UseBasicParsing
    
    # Execute the script content
    Write-Host "Executing setup script..." -ForegroundColor Green
    Invoke-Expression $setupScript
    
} catch {
    Write-Host "Error: Failed to download or execute setup script" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "Alternative: Clone the repository and run setup_silentwitness.bat" -ForegroundColor Yellow
    Write-Host "   Or check your internet connection and try again" -ForegroundColor Yellow
}
