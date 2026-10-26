$ErrorActionPreference = 'Stop'

$listener = Get-NetTCPConnection -LocalPort 3000 -State Listen -ErrorAction SilentlyContinue |
    Select-Object -First 1
if ($listener) {
    throw 'Port 3000 is already in use. Stop the running backend, then run this script again.'
}

$secureSecret = Read-Host 'Enter your NEW Cloudflare Turnstile secret key' -AsSecureString
if ($secureSecret.Length -eq 0) {
    throw 'The Turnstile secret key cannot be empty.'
}

$secretPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureSecret)
try {
    $env:TURNSTILE_SECRET_KEY = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($secretPointer)
}
finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($secretPointer)
    Remove-Variable secureSecret
}

$env:TURNSTILE_SITE_KEY = '0x4AAAAAAFnAH4F8qjf4Hmtn'

$ownsSmtpUsername = $false
$ownsSmtpPassword = $false
if ([string]::IsNullOrWhiteSpace($env:SMTP_USERNAME)) {
    $env:SMTP_USERNAME = Read-Host 'Gmail address used to send RustCord verification codes'
    $ownsSmtpUsername = $true
}
if ([string]::IsNullOrWhiteSpace($env:SMTP_APP_PASSWORD)) {
    $secureSmtpPassword = Read-Host 'Gmail app password (create one in your Google Account; not your normal password)' -AsSecureString
    if ($secureSmtpPassword.Length -eq 0) {
        throw 'Gmail app password is required for registration, sign-in, and password recovery.'
    }
    $smtpPasswordPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureSmtpPassword)
    try {
        $env:SMTP_APP_PASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($smtpPasswordPointer)
    }
    finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($smtpPasswordPointer)
        Remove-Variable secureSmtpPassword
    }
    $ownsSmtpPassword = $true
}

$ownsYouTubeApiKey = $false
if ([string]::IsNullOrWhiteSpace($env:YOUTUBE_API_KEY)) {
    $secureYouTubeApiKey = Read-Host 'YouTube Data API key (optional; press Enter to skip)' -AsSecureString
    if ($secureYouTubeApiKey.Length -gt 0) {
        $youtubeKeyPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureYouTubeApiKey)
        try {
            $env:YOUTUBE_API_KEY = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($youtubeKeyPointer)
            $ownsYouTubeApiKey = $true
        }
        finally {
            [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($youtubeKeyPointer)
            Remove-Variable secureYouTubeApiKey
        }
    }
    else {
        Remove-Variable secureYouTubeApiKey
    }
}

$ownsGiphyApiKey = $false
if ([string]::IsNullOrWhiteSpace($env:GIPHY_API_KEY)) {
    $secureGiphyApiKey = Read-Host 'Giphy public API key (browser-visible; optional, press Enter to skip)' -AsSecureString
    if ($secureGiphyApiKey.Length -gt 0) {
        $giphyKeyPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureGiphyApiKey)
        try {
            $env:GIPHY_API_KEY = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($giphyKeyPointer)
            $ownsGiphyApiKey = $true
        }
        finally {
            [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($giphyKeyPointer)
            Remove-Variable secureGiphyApiKey
        }
    }
    else {
        Remove-Variable secureGiphyApiKey
    }
}

Push-Location $PSScriptRoot
try {
    cargo run
    $cargoExitCode = $LASTEXITCODE
}
finally {
    Remove-Item Env:TURNSTILE_SECRET_KEY -ErrorAction SilentlyContinue
    Remove-Item Env:TURNSTILE_SITE_KEY -ErrorAction SilentlyContinue
    if ($ownsSmtpUsername) {
        Remove-Item Env:SMTP_USERNAME -ErrorAction SilentlyContinue
    }
    if ($ownsSmtpPassword) {
        Remove-Item Env:SMTP_APP_PASSWORD -ErrorAction SilentlyContinue
    }
    if ($ownsYouTubeApiKey) {
        Remove-Item Env:YOUTUBE_API_KEY -ErrorAction SilentlyContinue
    }
    if ($ownsGiphyApiKey) {
        Remove-Item Env:GIPHY_API_KEY -ErrorAction SilentlyContinue
    }
    Pop-Location
}

exit $cargoExitCode
