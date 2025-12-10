# Ouroboros Network - Lightweight Node Setup (Windows)
# Join the decentralized network in minutes (no database required!)

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "  🌐 Ouroboros Network - Quick Join" -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host ""

# Detect architecture
$arch = $env:PROCESSOR_ARCHITECTURE
switch ($arch) {
    "AMD64" { $binaryName = "ouro-node-windows-x64.exe" }
    "ARM64" { $binaryName = "ouro-node-windows-arm64.exe" }
    default {
        Write-Host "❌ Unsupported architecture: $arch" -ForegroundColor Red
        Write-Host "   Supported: AMD64 (x64), ARM64" -ForegroundColor Yellow
        exit 1
    }
}

# Create installation directory
$installDir = "$env:USERPROFILE\.ouroboros"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Set-Location $installDir

Write-Host "📥 Downloading Ouroboros node..." -ForegroundColor Yellow
Write-Host "   Architecture: $arch" -ForegroundColor Gray
Write-Host ""

# Download the latest release binary
$downloadUrl = "https://github.com/ipswyworld/ouroboros/releases/latest/download/$binaryName"
$outputPath = "$installDir\ouro-node.exe"

try {
    Write-Host "   Downloading from GitHub releases..." -ForegroundColor Gray
    Invoke-WebRequest -Uri $downloadUrl -OutFile $outputPath -UseBasicParsing
    Write-Host "✅ Binary downloaded successfully" -ForegroundColor Green
} catch {
    Write-Host "❌ Download failed: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "📦 Falling back to build from source..." -ForegroundColor Yellow
    Write-Host "   This requires:" -ForegroundColor Yellow
    Write-Host "   - Rust (https://rustup.rs/)" -ForegroundColor Yellow
    Write-Host "   - Git (https://git-scm.com/download/win)" -ForegroundColor Yellow
    Write-Host "   - LLVM (https://github.com/llvm/llvm-project/releases)" -ForegroundColor Yellow
    Write-Host ""

    # Check dependencies
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Rust not found. Please install from: https://rustup.rs/" -ForegroundColor Red
        Start-Process "https://rustup.rs/"
        exit 1
    }

    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Host "❌ Git not found. Please install from: https://git-scm.com/download/win" -ForegroundColor Red
        Start-Process "https://git-scm.com/download/win"
        exit 1
    }

    Write-Host "🔨 Building from source (this will take 15-30 minutes)..." -ForegroundColor Yellow

    # Clone and build
    Set-Location $env:TEMP
    if (Test-Path "ouroboros") {
        Remove-Item -Recurse -Force ouroboros
    }

    git clone https://github.com/ipswyworld/ouroboros.git
    Set-Location ouroboros\ouro_dag

    cargo build --release --bin ouro-node -j 2

    Copy-Item "target\release\ouro-node.exe" $outputPath
    Set-Location $installDir
}

Write-Host ""

# Get seed node address (allow override via environment variable)
$seedNode = if ($env:OUROBOROS_SEED) { $env:OUROBOROS_SEED } else { "136.112.101.176:9001" }

Write-Host "⚙️  Configuration:" -ForegroundColor Yellow
Write-Host "   Storage: RocksDB (lightweight, no database needed)" -ForegroundColor Gray
Write-Host "   Data directory: $installDir\data" -ForegroundColor Gray
Write-Host "   Seed node: $seedNode" -ForegroundColor Gray
Write-Host ""

# Create data directory
New-Item -ItemType Directory -Force -Path "$installDir\data" | Out-Null

# Create batch file for easy management
$batchContent = @"
@echo off
cd /d "$installDir"
ouro-node.exe join --peer $seedNode --storage rocksdb --rocksdb-path "$installDir\data" --api-port 8001 --p2p-port 9001
"@
$batchContent | Out-File -FilePath "$installDir\start-node.bat" -Encoding ASCII

# Create Windows service using NSSM or scheduled task
Write-Host "🚀 Starting Ouroboros node..." -ForegroundColor Yellow

# Start the node in background
$processArgs = "join --peer $seedNode --storage rocksdb --rocksdb-path `"$installDir\data`" --api-port 8001 --p2p-port 9001"
Start-Process -FilePath $outputPath -ArgumentList $processArgs -WindowStyle Hidden -RedirectStandardOutput "$installDir\node.log" -RedirectStandardError "$installDir\node_error.log"

Start-Sleep -Seconds 3

# Check if running
$process = Get-Process ouro-node -ErrorAction SilentlyContinue
if ($process) {
    Write-Host ""
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "✅ Node started successfully!" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "🌐 Connected to: $seedNode" -ForegroundColor Cyan
    Write-Host "💾 Storage: RocksDB (lightweight)" -ForegroundColor Cyan
    Write-Host "📂 Data directory: $installDir\data" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "📊 Check logs:" -ForegroundColor Yellow
    Write-Host "   Get-Content $installDir\node.log -Tail 50 -Wait" -ForegroundColor White
    Write-Host ""
    Write-Host "🔍 Check node status:" -ForegroundColor Yellow
    Write-Host "   curl http://localhost:8001/health" -ForegroundColor White
    Write-Host ""
    Write-Host "🛠️  Manage node:" -ForegroundColor Yellow
    Write-Host "   Restart: $installDir\start-node.bat" -ForegroundColor White
    Write-Host "   Stop: Get-Process ouro-node | Stop-Process" -ForegroundColor White
    Write-Host ""
    Write-Host "💡 Tip: To run node on startup, add start-node.bat to Windows Task Scheduler" -ForegroundColor Gray
    Write-Host ""
    Write-Host "🎉 You're now part of the Ouroboros network!" -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "❌ Error: Node failed to start" -ForegroundColor Red
    Write-Host "Check logs: Get-Content $installDir\node.log -Tail 50" -ForegroundColor Yellow
    Write-Host "Check errors: Get-Content $installDir\node_error.log -Tail 50" -ForegroundColor Yellow
    exit 1
}
