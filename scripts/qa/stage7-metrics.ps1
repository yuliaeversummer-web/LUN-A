param([int]$TargetPid, [string]$OutputFile, [int]$Seconds=1800, [string]$PythonExe)
if($TargetPid -le 0){throw 'A positive LUNÉA PID is required'}
$expected=(Resolve-Path "$PSScriptRoot/../../src-tauri/target").Path
if(-not (Get-Process -Id $TargetPid).Path.StartsWith($expected,[StringComparison]::OrdinalIgnoreCase)){throw 'Unexpected process path'}
$expectedStart=(Get-Process -Id $TargetPid).StartTime
$previousCpu = $null; $previousTime = $null
$until=(Get-Date).AddSeconds($Seconds)
do {
  $stamp=Get-Date
  $root=Get-Process -Id $TargetPid -ErrorAction SilentlyContinue
  if(-not $root -or $root.StartTime -ne $expectedStart){break}
  $inventory=Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe' OR Name='lunea.exe'" | Select-Object ProcessId,ParentProcessId
  $ids=@($TargetPid)
  do { $before=$ids.Count; foreach($item in $inventory){if($item.ParentProcessId -in $ids -and $item.ProcessId -notin $ids){$ids+= [int]$item.ProcessId}} } while($ids.Count -gt $before)
  $processes=@(Get-Process -Id $ids -ErrorAction SilentlyContinue)
  $cpu=[double]$root.CPU
  $row=[ordered]@{time=$stamp.ToString('o');pid=$TargetPid;cpuSeconds=$cpu;cpuPercentOneCore=if($null -ne $previousCpu){100*($cpu-$previousCpu)/($stamp-$previousTime).TotalSeconds}else{$null};workingSet=$root.WorkingSet64;privateBytes=$root.PrivateMemorySize64;handles=$root.HandleCount;threads=$root.Threads.Count;processCount=$processes.Count;treeWorkingSet=($processes | Measure-Object WorkingSet64 -Sum).Sum;treePrivateBytes=($processes | Measure-Object PrivateMemorySize64 -Sum).Sum;treeCpuSeconds=($processes | Measure-Object CPU -Sum).Sum;treeHandles=($processes | Measure-Object HandleCount -Sum).Sum;treeThreads=($processes | ForEach-Object {$_.Threads.Count} | Measure-Object -Sum).Sum}
  if($PythonExe){
    try {$row.workers=((& $PythonExe "$PSScriptRoot/stage7-threads.py" $TargetPid) | ConvertFrom-Json).workers} catch {$row.workerQueryError=$_.Exception.Message}
  }
  ($row | ConvertTo-Json -Depth 5 -Compress) | Add-Content -LiteralPath $OutputFile
  $previousCpu=$cpu; $previousTime=$stamp
  if((Get-Date) -ge $until){break}
  Start-Sleep -Seconds 30
} while($true)
