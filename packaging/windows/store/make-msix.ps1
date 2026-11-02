<#
  Builds the Microsoft Store package (MSIX) of flockswitch around a release build.

    packaging\windows\store\make-msix.ps1 -Exe target\release\flockswitch.exe -OutDir out\store `
        -IdentityName <Package/Identity/Name> -Publisher <CN=...> -PublisherDisplayName <name>

  The three identity values come from Partner Center → the app → Product management →
  Product identity, copied exactly. Without them a test identity is used: fine for trying
  the package on your own PC (test-signed), not for uploading to the Store.

  The package is not signed: the Store signs it after certification. Needs the Windows SDK
  (makeappx, makepri) and cargo, for the images.
#>
param(
    [Parameter(Mandatory = $true)] [string] $Exe,
    [Parameter(Mandatory = $true)] [string] $OutDir,
    [string] $IdentityName = "WWhaleFe.flockswitch.Test",
    [string] $Publisher = "CN=flockswitch test",
    [string] $PublisherDisplayName = "WWhaleFe"
)
$ErrorActionPreference = "Stop"
$root = (Resolve-Path "$PSScriptRoot\..\..\..").Path

# The newest Windows SDK's copy of a tool.
function Find-SdkTool([string] $name) {
    $found = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\$name" -ErrorAction SilentlyContinue |
        Sort-Object FullName -Descending | Select-Object -First 1
    if (-not $found) { throw "$name not found: install the Windows SDK" }
    $found.FullName
}

function Invoke-Tool([string] $tool, [string[]] $arguments) {
    & $tool @arguments
    if ($LASTEXITCODE -ne 0) { throw "$(Split-Path $tool -Leaf) failed ($LASTEXITCODE)" }
}

$version = (Select-String -Path "$root\Cargo.toml" -Pattern '^version = "(.+)"').Matches[0].Groups[1].Value
$packageVersion = "$version.0"
if ($version.StartsWith("0.")) {
    Write-Warning "Package version $packageVersion starts with 0, which the Store does not accept. Use 1.0.0 or later for a Store submission."
}

$OutDir = [System.IO.Path]::GetFullPath((Join-Path (Get-Location) $OutDir))
$layout = Join-Path $OutDir "layout"
if (Test-Path $layout) { Remove-Item -Recurse -Force $layout }
New-Item -ItemType Directory -Force (Join-Path $layout "Assets") | Out-Null
Copy-Item $Exe (Join-Path $layout "flockswitch.exe")

# Start tile, app list / taskbar icons and Store logo, in every scale and target size.
Push-Location $root
try { Invoke-Tool "cargo" @("run", "--quiet", "--release", "--example", "store-assets", "--", (Join-Path $layout "Assets")) }
finally { Pop-Location }

$manifest = Get-Content "$PSScriptRoot\AppxManifest.xml" -Raw
$values = [ordered]@{
    "@IDENTITY_NAME@"          = $IdentityName
    "@PUBLISHER_DISPLAY_NAME@" = $PublisherDisplayName
    "@PUBLISHER@"              = $Publisher
    "@VERSION@"                = $packageVersion
}
foreach ($key in $values.Keys) {
    $manifest = $manifest.Replace($key, [System.Security.SecurityElement]::Escape($values[$key]))
}
[System.IO.File]::WriteAllText((Join-Path $layout "AppxManifest.xml"), $manifest, (New-Object System.Text.UTF8Encoding $false))

# resources.pri lets Windows pick the right scale-/targetsize- image for each name the
# manifest uses.
$makepri = Find-SdkTool "makepri.exe"
$priconfig = Join-Path $OutDir "priconfig.xml"
Invoke-Tool $makepri @("createconfig", "/cf", $priconfig, "/dq", "en-US", "/pv", "10.0.0", "/o")
Invoke-Tool $makepri @("new", "/pr", $layout, "/cf", $priconfig, "/mn", (Join-Path $layout "AppxManifest.xml"), "/of", (Join-Path $layout "resources.pri"), "/o")

$makeappx = Find-SdkTool "makeappx.exe"
$msix = Join-Path $OutDir "flockswitch-$version-x64.msix"
Invoke-Tool $makeappx @("pack", "/d", $layout, "/p", $msix, "/o")
Write-Host "$msix ($IdentityName $packageVersion, $Publisher)"
