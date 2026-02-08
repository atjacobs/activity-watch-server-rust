@echo off
REM Generate API Key Hash for ActivityWatch
REM This is a simple alternative to the PowerShell script

echo.
echo ActivityWatch API Key Generator
echo ================================
echo.
echo Enter your API key (or leave blank for instructions to generate one):
set /p API_KEY=

if "%API_KEY%"=="" (
    echo.
    echo No API key provided. Here's how to generate one:
    echo.
    echo Option 1: Use PowerShell to generate a random key
    echo   powershell -Command "$bytes = New-Object byte[] 32; [Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($bytes); [Convert]::ToBase64String($bytes).Replace('+','').Replace('/','').Replace('=','').Substring(0,32)"
    echo.
    echo Option 2: Use an online generator
    echo   Visit: https://www.random.org/strings/?num=1^&len=32^&digits=on^&upperalpha=on^&loweralpha=on
    echo.
    echo Option 3: Make up your own (at least 16 characters, random)
    echo   Example: MyS3cr3tK3y2024!@#$
    echo.
    echo After generating a key, run this script again and paste it when prompted.
    echo.
    pause
    exit /b
)

echo.
echo API Key: %API_KEY%
echo.
echo Now generating SHA256 hash...
echo.

REM Generate hash using PowerShell
powershell -Command "$key = '%API_KEY%'; $bytes = [System.Text.Encoding]::UTF8.GetBytes($key); $hash = [System.Security.Cryptography.SHA256]::Create().ComputeHash($bytes); $hashString = [System.BitConverter]::ToString($hash).Replace('-','').ToLower(); Write-Host 'SHA256 Hash:'; Write-Host $hashString -ForegroundColor Green; Write-Host ''; Write-Host 'Add this to your config.toml:'; Write-Host ''; Write-Host '[security]'; Write-Host 'require_auth = true'; Write-Host 'allow_remote = true'; Write-Host ('api_keys = [\"' + $hashString + '\"]'); Write-Host ''; Write-Host 'Config file location:'; Write-Host ($env:APPDATA + '\activitywatch\aw-server-rust\config.toml');"

echo.
echo.
pause
