# SilentWitness One-Liner Installer
# Usage: irm https://raw.githubusercontent.com/GoblinRules/silentwitness/main/install_silentwitness.ps1 | iex

Write-Host "🎥 SilentWitness One-Click Installer" -ForegroundColor Cyan
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""

# Download and execute the main setup script
$setupScriptUrl = "https://raw.githubusercontent.com/GoblinRules/silentwitness/main/setup_silentwitness.ps1"

Write-Host "📥 Downloading SilentWitness setup script..." -ForegroundColor Yellow
Write-Host "   URL: $setupScriptUrl" -ForegroundColor Gray

try {
    # Download the setup script content
    $setupScript = Invoke-RestMethod -Uri $setupScriptUrl -UseBasicParsing
    
    # Execute the script content
    Write-Host "🚀 Executing setup script..." -ForegroundColor Green
    Invoke-Expression $setupScript
    
} catch {
    Write-Host "❌ Error: Failed to download or execute setup script" -ForegroundColor Red
    Write-Host "   Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 Alternative: Clone the repository and run setup_silentwitness.bat" -ForegroundColor Yellow
}

