# Generate API Key and Hash for ActivityWatch
# Usage: powershell -ExecutionPolicy Bypass -File .\generate-api-key.ps1 [ApiKey]
# If no ApiKey provided, generates a random one

param(
    [string]$ApiKey = ""
)

Write-Host ""
Write-Host "ActivityWatch API Key Generator" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

if ($ApiKey -eq "") {
    # Generate random key
    Write-Host "No API key provided. Generating a random secure key..." -ForegroundColor Yellow
    Write-Host ""

    $bytes = New-Object byte[] 32
    $rng = [Security.Cryptography.RandomNumberGenerator]::Create()
    $rng.GetBytes($bytes)
    $ApiKey = [Convert]::ToBase64String($bytes).Replace("+", "").Replace("/", "").Replace("=", "").Substring(0, 32)

    Write-Host "Generated API Key:" -ForegroundColor Green
    Write-Host $ApiKey -ForegroundColor Yellow
    Write-Host ""
    Write-Host "WARNING: Save this key securely! You will need it to access the server." -ForegroundColor Red
    Write-Host ""
} else {
    Write-Host "Using provided API key..." -ForegroundColor Yellow
    Write-Host ""
}

# Generate SHA256 hash
$utf8 = New-Object System.Text.UTF8Encoding
$bytes = $utf8.GetBytes($ApiKey)
$sha256 = [System.Security.Cryptography.SHA256]::Create()
$hashBytes = $sha256.ComputeHash($bytes)
$hashString = [System.BitConverter]::ToString($hashBytes).Replace("-", "").ToLower()

Write-Host "SHA256 Hash:" -ForegroundColor Green
Write-Host $hashString
Write-Host ""

Write-Host "Configuration Instructions:" -ForegroundColor Cyan
Write-Host "===========================" -ForegroundColor Cyan
Write-Host ""
Write-Host "1. Open your ActivityWatch config file:"
$configPath = "$env:APPDATA\activitywatch\aw-server-rust\config.toml"
Write-Host "   $configPath" -ForegroundColor Yellow
Write-Host ""

# Check if config directory exists
$configDir = Split-Path -Parent $configPath
if (-not (Test-Path $configDir)) {
    Write-Host "   Config directory doesn't exist. Creating it..." -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    Write-Host "   ✓ Created: $configDir" -ForegroundColor Green
    Write-Host ""
}

Write-Host "2. Add or update the [security] section:"
Write-Host ""
Write-Host "[security]" -ForegroundColor White
Write-Host "require_auth = true" -ForegroundColor White
Write-Host "allow_remote = true" -ForegroundColor White
Write-Host ('api_keys = ["' + $hashString + '"]') -ForegroundColor White
Write-Host ""
Write-Host "3. To allow remote connections, also update the address:"
Write-Host ""
Write-Host 'address = "0.0.0.0"  # Listen on all network interfaces' -ForegroundColor White
Write-Host ""
Write-Host "4. (Optional) Enable TLS for encrypted connections:"
Write-Host ""
Write-Host "[tls]" -ForegroundColor White
Write-Host "enabled = true" -ForegroundColor White
Write-Host 'cert = "C:\path\to\cert.pem"' -ForegroundColor White
Write-Host 'key = "C:\path\to\key.pem"' -ForegroundColor White
Write-Host ""
Write-Host "5. Restart the ActivityWatch server" -ForegroundColor Yellow
Write-Host ""

Write-Host "Client Usage:" -ForegroundColor Cyan
Write-Host "=============" -ForegroundColor Cyan
Write-Host "When connecting from a client, include the API key in your requests:"
Write-Host ""
Write-Host "  Authorization: Bearer $ApiKey" -ForegroundColor White
Write-Host ""
Write-Host "or" -ForegroundColor Gray
Write-Host ""
Write-Host "  X-API-Key: $ApiKey" -ForegroundColor White
Write-Host ""

# Offer to open config file
Write-Host "Would you like to open the config file now? (Y/N): " -ForegroundColor Yellow -NoNewline
$response = Read-Host

if (($response -eq "Y") -or ($response -eq "y")) {
    if (-not (Test-Path $configPath)) {
        Write-Host ""
        Write-Host "Config file does not exist. Creating it with template..." -ForegroundColor Yellow

        $template = "### ActivityWatch Server Configuration ###`n"
        $template += "`n"
        $template += "# Uncomment and modify the settings below as needed`n"
        $template += "`n"
        $template += "# Bind address (use 0.0.0.0 for all interfaces, 127.0.0.1 for localhost only)`n"
        $template += "#address = `"127.0.0.1`"`n"
        $template += "`n"
        $template += "# Port to listen on`n"
        $template += "#port = 5600`n"
        $template += "`n"
        $template += "# CORS origins (for web access)`n"
        $template += "#cors = []`n"
        $template += "`n"
        $template += "[security]`n"
        $template += "# Require API key authentication`n"
        $template += "require_auth = true`n"
        $template += "`n"
        $template += "# Allow remote access (required for non-localhost addresses)`n"
        $template += "allow_remote = true`n"
        $template += "`n"
        $template += "# API key hashes (add the hash generated above)`n"
        $template += "api_keys = [`"$hashString`"]`n"
        $template += "`n"
        $template += "[tls]`n"
        $template += "# Enable TLS/HTTPS`n"
        $template += "#enabled = false`n"
        $template += "`n"
        $template += "# Certificate file path`n"
        $template += "#cert = `"cert.pem`"`n"
        $template += "`n"
        $template += "# Private key file path`n"
        $template += "#key = `"key.pem`"`n"

        # Use Set-Content instead of Out-File for better compatibility
        Set-Content -Path $configPath -Value $template -Encoding UTF8
        Write-Host "Created config file with your API key hash" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "Opening config file..." -ForegroundColor Green
    Start-Process notepad $configPath
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host ""
