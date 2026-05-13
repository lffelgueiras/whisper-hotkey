# Downloads prebuilt llama.cpp binaries for Windows and places them in
# src-tauri/binaries/llama-runtime/. CI and local builds both call this
# before `pnpm tauri build`.
#
# Override the pinned release with $env:LLAMA_CPP_VERSION = 'bXXXX'.

$ErrorActionPreference = 'Stop'

$version = if ($env:LLAMA_CPP_VERSION) { $env:LLAMA_CPP_VERSION } else { 'b9131' }
$root = (Resolve-Path "$PSScriptRoot\..").Path
$dest = Join-Path $root 'src-tauri\binaries\llama-runtime'

$arch = $env:PROCESSOR_ARCHITECTURE
if (-not $arch) { $arch = (Get-CimInstance Win32_Processor).Architecture }

switch -Regex ($arch) {
    'ARM64|^12$' {
        # No Vulkan build for Windows arm64; CPU only.
        $asset = "llama-$version-bin-win-cpu-arm64.zip"
    }
    'AMD64|x86_64|^9$' {
        # Vulkan build includes CPU fallback DLLs.
        $asset = "llama-$version-bin-win-vulkan-x64.zip"
    }
    default {
        Write-Error "Unsupported Windows architecture: $arch"
    }
}

$url = "https://github.com/ggml-org/llama.cpp/releases/download/$version/$asset"
$tmp = New-TemporaryFile
$tmpZip = "$tmp.zip"
Remove-Item $tmp -Force

try {
    Write-Host "Fetching $url"
    Invoke-WebRequest -Uri $url -OutFile $tmpZip -UseBasicParsing

    if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
    New-Item -ItemType Directory -Path $dest | Out-Null

    Expand-Archive -Path $tmpZip -DestinationPath $dest -Force

    $count = (Get-ChildItem -Path $dest -File).Count
    Write-Host "Installed $count files to $dest"
} finally {
    if (Test-Path $tmpZip) { Remove-Item $tmpZip -Force }
}
