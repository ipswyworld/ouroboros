$Repo = "your-org/ouroboros"   # replace
$Tag = "v0.1.0"
$BinName = "ouro-node.exe"

$arch = if ($env:PROCESSOR_ARCHITECTURE -match "AMD64") { "x86_64" } else { "arm64" }
$os = "windows"
$url = "https://github.com/$Repo/releases/download/$Tag/$BinName-$os-$arch.zip"

Write-Host "Downloading $url"
Invoke-WebRequest -Uri $url -OutFile "$env:TEMP\$BinName.zip"

Expand-Archive -Path "$env:TEMP\$BinName.zip" -DestinationPath "$env:TEMP\ouro"
Move-Item -Path "$env:TEMP\ouro\$BinName" -Destination "$env:ProgramFiles\$BinName" -Force

# optionally add to PATH or instruct user to copy to a PATH folder
Write-Host "Installed $BinName to $env:ProgramFiles\$BinName"
Write-Host "Add to PATH or run from that folder."
