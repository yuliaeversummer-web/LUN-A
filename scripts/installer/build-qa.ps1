param([ValidateSet('0.1.0','0.1.1')][string]$Version='0.1.0', [switch]$ObserveUI)
$ErrorActionPreference='Stop'
$root=(Resolve-Path "$PSScriptRoot/../..").Path
$generated=Join-Path $root 'src-tauri/target/release/nsis/x64'
$qaDir=Join-Path $root 'output/stage-8/nsis-qa'
New-Item -ItemType Directory -Force $qaDir | Out-Null
Copy-Item "$generated/*.nsh" $qaDir -Force
$source=Get-Content -LiteralPath "$generated/installer.nsi" -Raw
foreach($item in @{
 'MANUFACTURER'='lunea-installer-qa'; 'PRODUCTNAME'='LUNEA-Installer-QA';
 'BUNDLEID'='app.lunea.installerqa'; 'VERSION'=$Version;
 'VERSIONWITHBUILD'="$Version.0";
 'OUTFILE'=(Join-Path $root "output/stage-8/qa-$Version-setup.exe")
}.GetEnumerator()){
 $pattern='(?m)^!define '+$item.Key+' ".*"$'
 if($source -notmatch $pattern){throw "Missing generated define $($item.Key)"}
 $source=[regex]::Replace($source,$pattern,[System.Text.RegularExpressions.MatchEvaluator]{param($match) '!define '+$item.Key+' "'+$item.Value+'"'})
}
Set-Content -LiteralPath "$qaDir/installer.nsi" -Value $source -Encoding utf8BOM
$qaFlags=@('/V2')
if($ObserveUI){$qaFlags+='/DLUNEA_QA_UI'}
& "$env:LOCALAPPDATA/tauri/NSIS/makensis.exe" @qaFlags "$qaDir/installer.nsi"
if($LASTEXITCODE -ne 0){throw 'QA NSIS compilation failed'}
Write-Output "Isolated QA installer built: $Version; no installation performed."

