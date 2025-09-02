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
$REPO_URL = "https://github.com/GoblinRules/silentwitness/archive/refs/heads/main.zip"

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



function Download-Repository {
    param([string]$OutputPath)
    
    try {
        Write-ColorOutput "Downloading SilentWitness repository..." "Info"
        Write-ColorOutput "   URL: $REPO_URL" "Info"
        
        $repoZip = Join-Path $OutputPath "silentwitness-main.zip"
        
        # Download repository
        Invoke-RestMethod -Uri $REPO_URL -OutFile $repoZip -UseBasicParsing
        
        if (Test-Path $repoZip) {
            $fileSize = [math]::Round((Get-Item $repoZip).Length / 1MB, 2)
            Write-ColorOutput "Downloaded repository successfully ($fileSize MB)" "Success"
            
            # Extract repository
            Write-ColorOutput "Extracting repository..." "Info"
            $extractPath = Join-Path $OutputPath "temp_extract"
            
            if (Test-Path $extractPath) {
                Remove-Item $extractPath -Recurse -Force
            }
            
            Expand-Archive -Path $repoZip -DestinationPath $extractPath -Force
            
            # Move contents from extracted folder to main directory
            $extractedFolder = Join-Path $extractPath "silentwitness-main"
            if (Test-Path $extractedFolder) {
                Write-ColorOutput "   Extracted folder found: $extractedFolder" "Info"
                
                # Copy all files and folders except .git directly to target
                $items = Get-ChildItem $extractedFolder | Where-Object { $_.Name -ne ".git" }
                Write-ColorOutput "   Found $($items.Count) items to copy:" "Info"
                
                foreach ($item in $items) {
                    $targetPath = Join-Path $OutputPath $item.Name
                    Write-ColorOutput "     Copying $($item.Name) to $targetPath" "Info"
                    
                    if ($item.PSIsContainer) {
                        Copy-Item $item.FullName -Destination $targetPath -Recurse -Force
                    } else {
                        Copy-Item $item.FullName -Destination $targetPath -Force
                    }
                }
                
                Write-ColorOutput "Repository content extracted successfully" "Success"
            } else {
                Write-ColorOutput "   Error: Extracted folder not found at $extractedFolder" "Error"
            }
            
            # Clean up
            Remove-Item $extractPath -Recurse -Force
            Remove-Item $repoZip -Force
            
            return $true
        } else {
            Write-ColorOutput "Failed to download repository" "Error"
            return $false
        }
    }
    catch {
        Write-ColorOutput "Error downloading repository - $($_.Exception.Message)" "Error"
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
        
        # Download file
        Invoke-RestMethod -Uri $Url -OutFile $OutputPath -UseBasicParsing
        
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
            
            # Install tkinter-embed FIRST (provides tkinter for embedded Python)
            Write-ColorOutput "   Installing tkinter-embed (provides tkinter support)..." "Info"
            & "$PythonPath\python.exe" -m pip install setuptools --no-warn-script-location --quiet
            & "$PythonPath\python.exe" -m pip install tkinter-embed --no-warn-script-location --quiet
            
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
        Write-ColorOutput "Error installing Python dependencies: $($_.Exception.Message)" "Error"
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
        Write-ColorOutput "Error configuring Python path: $($_.Exception.Message)" "Error"
        return $false
    }
}

# Main execution
Write-ColorOutput "SilentWitness Setup Script" "Info"
Write-ColorOutput "================================" "Info"

# Check if running as administrator
if (!(Test-Admin)) {
    Write-ColorOutput "Warning: Not running as administrator. Some operations may fail." "Warning"
    Write-ColorOutput "   Consider running PowerShell as Administrator for best results." "Warning"
    Write-ColorOutput ""
}

# Create main directory only
Write-ColorOutput "Creating main directory..." "Info"
Create-Directory $TOOLS_DIR

# Download and extract repository content
Write-ColorOutput ""
Write-ColorOutput "Downloading SilentWitness application..." "Info"
Write-ColorOutput "   This will download all scripts, configuration, and documentation" "Info"

# Clean up any existing directories that might conflict
$conflictingDirs = @("Scripts", "User Guide", "Startup", "README.md", "LICENSE.md")
foreach ($dir in $conflictingDirs) {
    $conflictPath = Join-Path $TOOLS_DIR $dir
    if (Test-Path $conflictPath) {
        Write-ColorOutput "   Removing existing $dir to avoid conflicts..." "Warning"
        Remove-Item $conflictPath -Recurse -Force
    }
}

$downloadSuccess = $false
$retryCount = 0
$maxRetries = 3

while (-not $downloadSuccess -and $retryCount -lt $maxRetries) {
    if ($retryCount -gt 0) {
        Write-ColorOutput "Retry attempt $retryCount of $maxRetries..." "Warning"
    }
    
    if (Download-Repository $TOOLS_DIR) {
        Write-ColorOutput "SilentWitness application downloaded successfully" "Success"
        $downloadSuccess = $true
    } else {
        $retryCount++
        if ($retryCount -lt $maxRetries) {
            Write-ColorOutput "Download failed, retrying in 5 seconds..." "Warning"
            Start-Sleep -Seconds 5
        } else {
            Write-ColorOutput "Failed to download application files after $maxRetries attempts" "Error"
            Write-ColorOutput "   You may need to manually copy the repository files" "Warning"
            Write-ColorOutput "   Or check your internet connection and try again" "Warning"
        }
    }
}

# Create additional directories needed by the application
Write-ColorOutput ""
Write-ColorOutput "Creating application directories..." "Info"
Create-Directory "$TOOLS_DIR\ffmpeg"
Create-Directory "$TOOLS_DIR\Python"
Create-Directory "$TOOLS_DIR\Recordings"
Create-Directory "$TOOLS_DIR\Logs"

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

# Download and setup embedded Python
if (!$SkipPython) {
    Write-ColorOutput ""
    Write-ColorOutput "Setting up embedded Python..." "Info"
    
    $pythonZip = Join-Path $TOOLS_DIR "python-3.12.0-embed-amd64.zip"
    
    if (Download-File -Url $PYTHON_URL -OutputPath $pythonZip -Description "Python") {
        if (Extract-Archive -ArchivePath $pythonZip -Destination "$TOOLS_DIR\Python" -Description "Python") {
            # Clean up zip file
            Remove-Item $pythonZip -Force
            
            # Configure Python and install dependencies
            if (Configure-PythonPath "$TOOLS_DIR\Python") {
                Install-PythonDependencies "$TOOLS_DIR\Python"
            }
            
            Write-ColorOutput "Embedded Python setup completed successfully" "Success"
        }
    }
} elseif ($SkipPython) {
    Write-ColorOutput "Skipping Python setup" "Warning"
}

# Verify repository content
Write-ColorOutput ""
Write-ColorOutput "Verifying installation..." "Info"

$requiredFiles = @(
    "Scripts\ffmpeg_auto_recorder.py",
    "Scripts\recorder_status_gui.py",
    "Scripts\config.ini",
    "README.md",
    "LICENSE.md"
)

$missingFiles = @()
foreach ($file in $requiredFiles) {
    $filePath = Join-Path $TOOLS_DIR $file
    if (Test-Path $filePath) {
        Write-ColorOutput "   $file" "Success"
    } else {
        Write-ColorOutput "   $file (missing)" "Error"
        $missingFiles += $file
    }
}

if ($missingFiles.Count -eq 0) {
    Write-ColorOutput "All required files are present" "Success"
} else {
    Write-ColorOutput "Warning: Some files are missing - installation may be incomplete" "Warning"
}

# Show final directory structure
Write-ColorOutput ""
Write-ColorOutput "Final directory structure:" "Info"
Get-ChildItem $TOOLS_DIR | ForEach-Object {
    if ($_.PSIsContainer) {
        $itemCount = (Get-ChildItem $_.FullName -Recurse | Measure-Object).Count
        Write-ColorOutput "   Folder: $($_.Name) ($itemCount items)" "Success"
    } else {
        Write-ColorOutput "   File: $($_.Name)" "Success"
    }
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
} else {
    Write-ColorOutput "Configuration file not found - repository may not have downloaded correctly" "Warning"
}

# Setup startup options if requested
Write-ColorOutput ""
Write-ColorOutput "Startup configuration..." "Info"
Write-ColorOutput "You can configure startup options using:" "Info"
Write-ColorOutput "  1. Run: python Scripts\ini_editor.py" "Info"
Write-ColorOutput "  2. Or use: python Scripts\startup_manager.py --help" "Info"
Write-ColorOutput "  3. Or double-click: Scripts\manage_startup.bat" "Info"

Write-ColorOutput ""
Write-ColorOutput "SilentWitness setup completed!" "Success"
Write-ColorOutput ""
Write-ColorOutput "Installation directory: $TOOLS_DIR" "Info"
Write-ColorOutput "To start using SilentWitness:" "Info"
Write-ColorOutput "   1. cd $TOOLS_DIR\Scripts" "Info"
Write-ColorOutput "   2. python ffmpeg_auto_recorder.py" "Info"
Write-ColorOutput ""
Write-ColorOutput "What was installed:" "Info"
Write-ColorOutput "   FFmpeg for video recording" "Success"
Write-ColorOutput "   Embedded Python 3.12 with tkinter support" "Success"
Write-ColorOutput "   SilentWitness application files" "Success"
Write-ColorOutput "   Configuration and documentation" "Success"
Write-ColorOutput ""
Write-ColorOutput "See README.md for complete usage instructions" "Info"
Write-ColorOutput ""
Write-ColorOutput "Note: All files are now in $TOOLS_DIR - no need to clone the repository!" "Info"

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
