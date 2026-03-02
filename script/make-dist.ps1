# make-dist.ps1 - Assemble distribution directory
# Usage: powershell -ExecutionPolicy Bypass -File script\make-dist.ps1

param(
    [string]$ProjectDir = (Split-Path -Parent $PSScriptRoot)
)

Set-Location $ProjectDir
Write-Host "=== Creating dist directory ===" -ForegroundColor Cyan
Write-Host "Project: $ProjectDir"

# Clean and create dist directory
$distDir = Join-Path $ProjectDir "dist"
if (Test-Path $distDir) {
    Remove-Item -Recurse -Force "$distDir\*" -ErrorAction SilentlyContinue
}
New-Item -ItemType Directory -Force -Path $distDir | Out-Null
New-Item -ItemType Directory -Force -Path "$distDir\config" | Out-Null
New-Item -ItemType Directory -Force -Path "$distDir\data" | Out-Null

# 1. Copy executable
$exePath = Join-Path $ProjectDir "target\release\dm-rust.exe"
if (Test-Path $exePath) {
    Copy-Item $exePath "$distDir\dm-rust.exe"
    $size = [math]::Round((Get-Item $exePath).Length / 1MB, 2)
    Write-Host "[OK] dm-rust.exe copied ($size MB)" -ForegroundColor Green
} else {
    Write-Host "[WARN] dm-rust.exe not found - run: cargo build --release" -ForegroundColor Yellow
}

# 2. Copy dist-config (Vue config UI)
$distConfigDir = Join-Path $ProjectDir "dist-config"
if (Test-Path $distConfigDir) {
    Copy-Item -Recurse $distConfigDir "$distDir\dist-config"
    Write-Host "[OK] dist-config/ copied (config UI)" -ForegroundColor Green
} else {
    Write-Host "[WARN] dist-config/ not found - run: cd config-ui && npm run build" -ForegroundColor Yellow
}

# 3. Copy config files
$configSrc = Join-Path $ProjectDir "config\config1.json"
if (Test-Path $configSrc) {
    Copy-Item $configSrc "$distDir\config\config.json"
    Write-Host "[OK] config1.json -> config/config.json" -ForegroundColor Green
}

$configExample = Join-Path $ProjectDir "config\config.example.json"
if (Test-Path $configExample) {
    Copy-Item $configExample "$distDir\config\config.example.json"
    Write-Host "[OK] config.example.json copied" -ForegroundColor Green
}

# 4. Copy migrations
$migrationsDir = Join-Path $ProjectDir "migrations"
if (Test-Path $migrationsDir) {
    Copy-Item -Recurse $migrationsDir "$distDir\migrations"
    Write-Host "[OK] migrations/ copied" -ForegroundColor Green
}

# Show final structure
Write-Host ""
Write-Host "=== dist/ structure ===" -ForegroundColor Cyan
function Show-Tree {
    param([string]$Path, [string]$Prefix = "")
    $items = Get-ChildItem -Path $Path | Sort-Object -Property PSIsContainer -Descending
    for ($i = 0; $i -lt $items.Count; $i++) {
        $item = $items[$i]
        $isLast = ($i -eq $items.Count - 1)
        $connector = if ($isLast) { "L-- " } else { "|-- " }
        if ($item.PSIsContainer) {
            Write-Host "$Prefix$connector$($item.Name)/" -ForegroundColor Blue
            $newPrefix = if ($isLast) { "$Prefix    " } else { "$Prefix|   " }
            Show-Tree -Path $item.FullName -Prefix $newPrefix
        } else {
            $size = if ($item.Length -gt 1MB) {
                "{0:N2} MB" -f ($item.Length / 1MB)
            } elseif ($item.Length -gt 1KB) {
                "{0:N1} KB" -f ($item.Length / 1KB)
            } else {
                "$($item.Length) B"
            }
            Write-Host "$Prefix$connector$($item.Name)  ($size)" -ForegroundColor White
        }
    }
}
Show-Tree -Path $distDir

Write-Host ""
Write-Host "=== Done! ===" -ForegroundColor Cyan
Write-Host "To run: .\dist\dm-rust.exe --config .\dist\config\config.json"
