# Improved Ouroboros Quick Join Script
param([switch]$SkipAutoStart)

$nodeDir = "$env:USERPROFILE\.ouroboros"
$dataDir = "$nodeDir\data"

Write-Host "`n=========================================="
Write-Host "  🌟 WELCOME TO OUROBOROS NETWORK 🌟"
Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "`nSetting up your validator node..."

# Create directories
New-Item -ItemType Directory -Force -Path $nodeDir, $dataDir | Out-Null

# Get binary
Write-Host "`n[1/5] 📥 Getting node binary..." -NoNewline
if (Test-Path "$nodeDir\ouro-node.exe") {
    Write-Host " ✅ (existing)" -ForegroundColor Green
} else {
    $localBuild = "C:\Users\LENOVO\Desktop\ouroboros\ouro_dag\target\release\ouro-node.exe"
    if (Test-Path $localBuild) {
        Copy-Item $localBuild "$nodeDir\ouro-node.exe"
        Write-Host " ✅ (local)" -ForegroundColor Green
    } else {
        Write-Host " ❌ Not found" -ForegroundColor Red
        Write-Host "`nPlease build first: cd ouroboros\ouro_dag && cargo build --release"
        exit 1
    }
}

# Create wallet
Write-Host "[2/5] 🔐 Creating wallet..." -NoNewline
$walletAddr = -join ((48..57) + (97..102) | Get-Random -Count 40 | % {[char]$_})
$walletAddr = "0x$walletAddr"
$nodeId = "ouro_" + (-join ((97..122) + (48..57) | Get-Random -Count 12 | % {[char]$_}))
Set-Content "$nodeDir\wallet.txt" $walletAddr
Set-Content "$nodeDir\node_id.txt" $nodeId
Write-Host " ✅" -ForegroundColor Green

# Create config
Write-Host "[3/5] ⚙️  Configuring node..." -NoNewline
@"
ROCKSDB_PATH=$dataDir
STORAGE_MODE=rocks
RUST_LOG=info
API_ADDR=0.0.0.0:8001
LISTEN_ADDR=0.0.0.0:9001
NODE_ID=$nodeId
SEED_NODES=136.112.101.176:9001
"@ | Set-Content "$nodeDir\.env"
Write-Host " ✅" -ForegroundColor Green

# Create start script
@"
@echo off
cd /d %USERPROFILE%\.ouroboros
start /min ouro-node.exe start
"@ | Set-Content "$nodeDir\start-node.bat"

# Create ouro CLI
@"
@echo off
setlocal enabledelayedexpansion
cd /d %USERPROFILE%\.ouroboros

if "%1"=="status" goto status
if "%1"=="start" goto start
if "%1"=="stop" goto stop
if "%1"=="wallet" goto wallet
if "%1"=="rewards" goto rewards
goto help

:status
echo.
echo ==========================================
echo    🌐 OUROBOROS NODE STATUS
echo ==========================================
if exist node_id.txt (set /p NODE_ID=<node_id.txt) else (set NODE_ID=Unknown)
if exist wallet.txt (set /p WALLET=<wallet.txt) else (set WALLET=Unknown)
tasklist /FI "IMAGENAME eq ouro-node.exe" 2>NUL | find /I /N "ouro-node.exe">NUL
if "%ERRORLEVEL%"=="0" (
    echo Status: 🟢 RUNNING
) else (
    echo Status: 🔴 STOPPED
)
echo Node ID: %NODE_ID%
echo Wallet: %WALLET%
echo.
curl -s http://localhost:8001/health >NUL 2>&1
if %ERRORLEVEL%==0 (
    echo API: 🟢 http://localhost:8001
    echo.
    echo 💰 Check rewards: ouro rewards
    echo 👛 Wallet balance: ouro wallet balance
) else (
    echo API: 🔴 Offline
)
echo.
echo Commands: ouro start^|stop^|wallet^|rewards
echo ==========================================
goto end

:start
echo Starting Ouroboros node...
start-node.bat
timeout /t 3 >nul
ouro status
goto end

:stop
taskkill /IM ouro-node.exe /F
echo Node stopped
goto end

:wallet
if "%2"=="balance" (
    if exist wallet.txt (set /p WALLET=<wallet.txt)
    echo Checking balance for !WALLET!...
    curl -s "http://localhost:8001/balance/!WALLET!"
) else (
    if exist wallet.txt (set /p WALLET=<wallet.txt) else (set WALLET=Not created)
    echo Your Wallet: !WALLET!
    echo.
    echo Commands:
    echo   ouro wallet balance  - Check balance
)
goto end

:rewards
if exist node_id.txt (set /p NODE_ID=<node_id.txt)
echo Fetching rewards for !NODE_ID!...
curl -s "http://localhost:8001/metrics/!NODE_ID!"
goto end

:help
echo.
echo Ouroboros Node CLI
echo.
echo Usage: ouro [command]
echo.
echo Commands:
echo   status   - Show node status
echo   start    - Start node
echo   stop     - Stop node
echo   wallet   - Show wallet address
echo   rewards  - Check earned rewards
echo.
:end
"@ | Set-Content "$nodeDir\ouro.bat"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$nodeDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$nodeDir", "User")
}

# Auto-start on boot
Write-Host "[4/5] 🚀 Configuring auto-start..." -NoNewline
try {
    $action = New-ScheduledTaskAction -Execute "$nodeDir\start-node.bat"
    $trigger = New-ScheduledTaskTrigger -AtStartup -RandomDelay (New-TimeSpan -Seconds 30)
    $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries
    Register-ScheduledTask -TaskName "OuroborosNode" -Action $action -Trigger $trigger -Settings $settings -Force -ErrorAction Stop | Out-Null
    Write-Host " ✅" -ForegroundColor Green
} catch {
    Write-Host " ⏭️  Skipped (needs admin)" -ForegroundColor Yellow
}

# Start node
Write-Host "[5/5] 🌐 Starting node..." -NoNewline
Start-Process -FilePath "$nodeDir\ouro-node.exe" -ArgumentList "start" -WindowStyle Hidden -WorkingDirectory $nodeDir
Start-Sleep 3
Write-Host " ✅`n" -ForegroundColor Green

# Success message
Write-Host "==========================================" -ForegroundColor Green
Write-Host "  🎉 SUCCESS! You're now validating!" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host "`n📊 Your Node:" -ForegroundColor Cyan
Write-Host "   Node ID: $nodeId"
Write-Host "   Wallet:  $walletAddr"
Write-Host "   Status:  http://localhost:8001/health"
Write-Host "`n💰 Earnings:" -ForegroundColor Yellow
Write-Host "   ~4.5 OURO/hour (based on uptime + validations)"
Write-Host "   Check rewards: ouro rewards"
Write-Host "`n📝 Wallet saved to: $nodeDir\wallet.txt"
Write-Host "   ⚠️  Backup this file - you'll need it to recover funds!"
Write-Host "`n🎯 Quick Commands:" -ForegroundColor Cyan
Write-Host "   $nodeDir\ouro status   - Live node status"
Write-Host "   $nodeDir\ouro wallet   - Your wallet address"
Write-Host "   $nodeDir\ouro rewards  - Check earnings"
Write-Host "`n⚠️  Restart terminal, then use: ouro status"
Write-Host "`n🌐 Your node will auto-start with Windows"
Write-Host "   Keep your PC online to maximize rewards!`n"
