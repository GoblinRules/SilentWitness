# SilentWitness Setup Script
# Downloads and configures FFmpeg and Python environment
# Uses Invoke-RestMethod (IRM) and Invoke-Expression (IEX) for security

param(
    [switch]$Force,
    [switch]$SkipFFmpeg,
    [switch]$SkipPython
)

# Set execution policy for this session
Set-ExecutionPolicy -ExecutionPolicy Bypass -Scope Process -Force

# Configuration
$TOOLS_DIR = "C:\Tools\SilentWitness"
$FFMPEG_URL = "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip"
$PYTHON_URL = "https://www.python.org/ftp/python/3.12.0/python-3.12.0-embed-amd64.zip"
$PIP_URL = "https://bootstrap.pypa.io/get-pip.py"

# Colors for output
$Colors = @{
    Info = "Cyan"
    Success = "Green"
    Warning = "Yellow"
    Error = "Red"
}

function Write-ColorOutput {
    param([string]$Message, [string]$Color = "White")
    
    # Fallback to white if color lookup fails
    try {
        $colorValue = $Colors[$Color]
        if ($colorValue) {
            Write-Host $Message -ForegroundColor $colorValue
        } else {
            Write-Host $Message
        }
    }
    catch {
        # If anything goes wrong, just write without color
        Write-Host $Message
    }
}

function Test-Admin {
    try {
        $currentUser = [Security.Principal.WindowsIdentity]::GetCurrent()
        $principal = New-Object Security.Principal.WindowsPrincipal($currentUser)
        return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
    }
    catch {
        return $false
    }
}

function Create-Directory {
    param([string]$Path)
    if (!(Test-Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        Write-ColorOutput "Created directory: $Path" "Success"
    }
}

function Download-File {
    param([string]$Url, [string]$OutputPath, [string]$Description)
    
    try {
        Write-ColorOutput "Downloading $Description..." "Info"
        Write-ColorOutput "   URL: $Url" "Info"
        
        # Show download progress
        $progressParams = @{
            Uri = $Url
            OutFile = $OutputPath
            UseBasicParsing = $true
            ProgressAction = {
                $percentComplete = $_.PercentComplete
                if ($percentComplete -ge 0) {
                    Write-Progress -Activity "Downloading $Description" -Status "$percentComplete% Complete" -PercentComplete $percentComplete
                }
            }
        }
        
        Invoke-RestMethod @progressParams
        
        # Clear progress bar
        Write-Progress -Activity "Downloading $Description" -Completed
        
        if (Test-Path $OutputPath) {
            $fileSize = [math]::Round((Get-Item $OutputPath).Length / 1MB, 2)
            Write-ColorOutput "Downloaded $Description successfully ($fileSize MB)" "Success"
            return $true
        } else {
            Write-ColorOutput "Failed to download $Description" "Error"
            return $false
        }
    }
    catch {
        Write-ColorOutput "Error downloading $Description - $($_.Exception.Message)" "Error"
        return $false
    }
}

function Extract-Archive {
    param([string]$ArchivePath, [string]$Destination, [string]$Description)
    
    try {
        Write-ColorOutput "Extracting $Description..." "Info"
        
        # Use Expand-Archive for better compatibility
        Expand-Archive -Path $ArchivePath -DestinationPath $Destination -Force
        
        Write-ColorOutput "Extracted $Description successfully" "Success"
        return $true
    }
    catch {
        Write-ColorOutput "Error extracting $Description - $($_.Exception.Message)" "Error"
        return $false
    }
}

function Install-PythonDependencies {
    param([string]$PythonPath)
    
    try {
        Write-ColorOutput "Installing Python dependencies..." "Info"
        
        # Download get-pip.py
        $pipPath = Join-Path $TOOLS_DIR "get-pip.py"
        if (Download-File -Url $PIP_URL -OutputPath $pipPath -Description "get-pip.py") {
            
            Write-ColorOutput "   Installing pip..." "Info"
            & "$PythonPath\python.exe" $pipPath --no-warn-script-location --quiet
            
            # Install required packages
            $requirements = @(
                "pyautogui",
                "pymsgbox", 
                "pygetwindow",
                "pytweening",
                "pyscreeze",
                "pyrect",
                "pywin32",
                "pyperclip",
                "mouse",
                "pystray",
                "pillow",
                "screeninfo",
                "pygame",
                "pynput",
                "configparser",
                "psutil"
            )
            
            $totalPackages = $requirements.Count
            $currentPackage = 0
            
            foreach ($package in $requirements) {
                $currentPackage++
                Write-ColorOutput "   Installing package $currentPackage of $totalPackages - $package" "Info"
                
                # Show pip progress
                $pipOutput = & "$PythonPath\python.exe" -m pip install $package --no-warn-script-location --quiet 2>&1
                if ($LASTEXITCODE -eq 0) {
                    Write-ColorOutput "     $package installed" "Success"
                } else {
                    Write-ColorOutput "     Warning: $package had issues" "Warning"
                }
            }
            
            Write-ColorOutput "Python dependencies installation completed" "Success"
            return $true
        }
    }
    catch {
        Write-ColorOutput "Error installing Python dependencies - $($_.Exception.Message)" "Error"
        return $false
    }
}

function Configure-PythonPath {
    param([string]$PythonPath)
    
    try {
        Write-ColorOutput "Configuring Python path..." "Info"
        
        # Create python312._pth file
        $pthContent = @"
python312.zip
.

# Uncomment to run site.main() automatically
import site
Lib\site-packages
"@
        
        $pthPath = Join-Path $PythonPath "python312._pth"
        $pthContent | Out-File -FilePath $pthPath -Encoding ASCII
        
        Write-ColorOutput "Python path configured successfully" "Success"
        return $true
    }
    catch {
        Write-ColorOutput "Error configuring Python path - $($_.Exception.Message)" "Error"
        return $false
    }
}

# Main execution
try {
    Write-ColorOutput "SilentWitness Setup Script" "Info"
    Write-ColorOutput "================================" "Info"
} catch {
    Write-Host "SilentWitness Setup Script"
    Write-Host "================================"
}

# Check if running as administrator
try {
    if (!(Test-Admin)) {
        Write-ColorOutput "Warning: Not running as administrator. Some operations may fail." "Warning"
        Write-ColorOutput "   Consider running PowerShell as Administrator for best results." "Warning"
        Write-ColorOutput ""
    }
} catch {
    Write-Host "Warning: Could not determine administrator status. Some operations may fail."
    Write-Host ""
}

# Create main directory structure
Write-ColorOutput "Creating directory structure..." "Info"
Create-Directory $TOOLS_DIR
Create-Directory "$TOOLS_DIR\Scripts"
Create-Directory "$TOOLS_DIR\ffmpeg"
Create-Directory "$TOOLS_DIR\Python"
Create-Directory "$TOOLS_DIR\Recordings"
Create-Directory "$TOOLS_DIR\Logs"
Create-Directory "$TOOLS_DIR\Startup\Recorder"

# Download and setup FFmpeg
if (!$SkipFFmpeg) {
    Write-ColorOutput ""
    Write-ColorOutput "Setting up FFmpeg..." "Info"
    
    $ffmpegZip = Join-Path $TOOLS_DIR "ffmpeg-release-essentials.zip"
    
    if (Download-File -Url $FFMPEG_URL -OutputPath $ffmpegZip -Description "FFmpeg") {
        if (Extract-Archive -ArchivePath $ffmpegZip -Destination "$TOOLS_DIR\ffmpeg" -Description "FFmpeg") {
            # Move contents from subfolder to main ffmpeg directory
            $ffmpegSubdir = Get-ChildItem "$TOOLS_DIR\ffmpeg" -Directory | Where-Object { $_.Name -like "*ffmpeg*" } | Select-Object -First 1
            if ($ffmpegSubdir) {
                Move-Item "$($ffmpegSubdir.FullName)\*" "$TOOLS_DIR\ffmpeg\" -Force
                Remove-Item $ffmpegSubdir.FullName -Force
            }
            
            # Clean up zip file
            Remove-Item $ffmpegZip -Force
            Write-ColorOutput "FFmpeg setup completed successfully" "Success"
        }
    }
} else {
    Write-ColorOutput "Skipping FFmpeg setup" "Warning"
}

# Download and setup Python
if (!$SkipPython) {
    Write-ColorOutput ""
    Write-ColorOutput "Setting up Python..." "Info"
    
    $pythonZip = Join-Path $TOOLS_DIR "python-3.12.0-embed-amd64.zip"
    
    if (Download-File -Url $PYTHON_URL -OutputPath $pythonZip -Description "Python") {
        if (Extract-Archive -ArchivePath $pythonZip -Destination "$TOOLS_DIR\Python" -Description "Python") {
            # Clean up zip file
            Remove-Item $pythonZip -Force
            
            # Configure Python and install dependencies
            if (Configure-PythonPath "$TOOLS_DIR\Python") {
                Install-PythonDependencies "$TOOLS_DIR\Python"
            }
            
            Write-ColorOutput "Python setup completed successfully" "Success"
        }
    }
} else {
    Write-ColorOutput "Skipping Python setup" "Warning"
}

# Final configuration
Write-ColorOutput ""
Write-ColorOutput "Final configuration..." "Info"

# Update config.ini paths if it exists
$configPath = Join-Path $TOOLS_DIR "Scripts\config.ini"
if (Test-Path $configPath) {
    Write-ColorOutput "Updating configuration paths..." "Info"
    
    $configContent = Get-Content $configPath -Raw
    $configContent = $configContent -replace "C:/Tools/OBS", "C:/Tools/SilentWitness"
    $configContent = $configContent -replace "C:\\Tools\\OBS", "C:\Tools\SilentWitness"
    $configContent | Out-File $configPath -Encoding UTF8
    
    Write-ColorOutput "Configuration updated successfully" "Success"
}

Write-ColorOutput ""
Write-ColorOutput "SilentWitness setup completed!" "Success"
Write-ColorOutput ""
Write-ColorOutput "Installation directory: $TOOLS_DIR" "Info"
Write-ColorOutput "To start using SilentWitness:" "Info"
Write-ColorOutput "   1. cd $TOOLS_DIR\Scripts" "Info"
Write-ColorOutput "   2. python ffmpeg_auto_recorder.py" "Info"
Write-ColorOutput ""
Write-ColorOutput "See README.md for complete usage instructions" "Info"
Write-ColorOutput ""
Write-ColorOutput "Note: Ensure all paths in config.ini are correct for your system" "Warning"
