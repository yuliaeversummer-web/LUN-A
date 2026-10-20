import { readFile, writeFile } from 'node:fs/promises'
const lines=async path=>(await readFile(path,'utf8')).trim().split(/\r?\n/).filter(Boolean).map(line=>JSON.parse(line))
const baseline=await lines('output/stage-7/baseline-hidden-valid.jsonl')
const final=await lines('output/stage-7/cold-final.jsonl')
const soakPath=process.argv[2] || 'output/stage-7/soak-resume-summary.json'
const soak=JSON.parse(await readFile(soakPath,'utf8'))
const installer=JSON.parse(await readFile('output/stage-7/installer.json','utf8'))
const duration=rows=>(Date.parse(rows.at(-1).time)-Date.parse(rows[0].time))/1000
const cpu=(rows,key)=>100*(rows.at(-1)[key]-rows[0][key])/duration(rows)
const range=(rows,key,scale=1)=>{const values=rows.map(row=>row[key]/scale);return `${Math.min(...values).toFixed(scale===1?0:1)}–${Math.max(...values).toFixed(scale===1?0:1)}`}
const number=(value,digits=2)=>Number.isFinite(value)?value.toFixed(digits):'нет достаточных данных'
const mib=1048576,oneCore=(value)=>value.toFixed(2)+'%',machine=value=>(value/12).toFixed(2)+'%'
let report=`## Итоговые измерения\n\nBaseline: исходный release этапа 6, оба окна скрыты, ${duration(baseline).toFixed(2)}с, ${baseline.length} samples.\nFinal: тот же сценарий после прогрева нового release, ${duration(final).toFixed(2)}с, ${final.length} samples.\nОба сравнения с локальным CDP; не строгий лабораторный benchmark, внешние приложения продолжали работать.\nCPU ниже — процент одного логического ядра; на данном ПК 12 logical processors.\n\n| Метрика | Baseline | Final |\n|---|---:|---:|\n`
for(const [name,b,f] of [
 ['CPU lunea.exe',oneCore(cpu(baseline,'cpuSeconds')),oneCore(cpu(final,'cpuSeconds'))],
 ['CPU lunea.exe в нормировке Task Manager',machine(cpu(baseline,'cpuSeconds')),machine(cpu(final,'cpuSeconds'))],
 ['Working Set lunea.exe, MiB',range(baseline,'workingSet',mib),range(final,'workingSet',mib)],
 ['Private Bytes lunea.exe, MiB',range(baseline,'privateBytes',mib),range(final,'privateBytes',mib)],
 ['Handles lunea.exe',range(baseline,'handles'),range(final,'handles')],
 ['Threads lunea.exe',range(baseline,'threads'),range(final,'threads')],
 ['CPU дерево LUNÉA + WebView2',oneCore(cpu(baseline,'treeCpuSeconds')),oneCore(cpu(final,'treeCpuSeconds'))],
 ['Сумма Private Bytes дерева, MiB',range(baseline,'treePrivateBytes',mib),range(final,'treePrivateBytes',mib)],
 ['Processes в дереве',range(baseline,'processCount'),range(final,'processCount')],
])report+=`| ${name} | ${b} | ${f} |\n`
report+=`\nСумма Working Set процессов может повторно учитывать shared pages; Private Bytes и память самого lunea.exe приведены отдельно.\nВ первоначальном 120-секундном sampler CPU было ошибочно посчитано через обновляемый Process object; корректные delta восстановлены из cpuSeconds/time в baseline-corrected.json.\nФайл baseline-hidden.jsonl с PID=0 исключён; используется только baseline-hidden-valid.jsonl с проверенным PID=20736.\nBaseline command latency отдельно не сохранена, ускорение команд не заявляется.\n\n## Непрерывный soak\n\nДлительность UI-сценария: **${soak.uiDurationSeconds.toFixed(1)}с**; серия процесса ${soak.durationSeconds.toFixed(1)}с, ${soak.samples} samples, интервал около 30с.\nПрогон остановлен по просьбе пользователя: полный 30-минутный тест отменён. Первые 3 минуты исключены из steady агрегатов как прогрев; поэтому в этом коротком прогоне steady hidden данных недостаточно. Отдельный прежний прогон 23 мин 16 с не объединяется с новым; его summary сохранён в soak-30-summary.json. В процессе выполнялись циклы main/overlay/both hidden и только cached state reads, без управления звуком/Discord/воспроизведением.\n\n| lunea.exe | Начало → конец | Диапазон |\n|---|---:|---:|\n`
for(const [key,name,scale]of [['workingSet','Working Set, MiB',mib],['privateBytes','Private Bytes, MiB',mib],['handles','Handles',1],['threads','Threads',1]]){
 const v=soak.root[key],digits=scale===1?0:1
 report+=`| ${name} | ${(v.first/scale).toFixed(digits)} → ${(v.last/scale).toFixed(digits)} | ${(v.min/scale).toFixed(digits)}–${(v.max/scale).toFixed(digits)} |\n`
}
report+=`\nВо всех samples по одному lunea-core-audio / lunea-gsmtc / lunea-discord: **${soak.workersAlwaysOne}**.\nPage errors: ${soak.pageErrors.length}; fatal сценария: ${soak.fatal.length}; Discord оставался unknown: ${soak.discordRemainedUnknown}.\nСкрытые устойчивые интервалы: ${soak.hidden.seconds.toFixed(1)}с; media mutations максимум ${number(soak.hidden.maxMediaMutations,0)} за 30с.\nВидимые media updates: median ${number(soak.visibleMediaUpdatesPerSecond.median)}/с; maximum revision lag ${soak.maxVisibleMediaRevisionLag}.\nЧастота revisions зависит от пользовательского воспроизведения; paused не считается остановкой синхронизации.\nBackend source IDs: ${soak.sources.join(', ')}.\n\n| Cached state read | Median, ms | P95, ms | Max, ms |\n|---|---:|---:|---:|\n`
for(const [name,s]of Object.entries(soak.perService))report+=`| ${name} | ${s.latencyMs.median.toFixed(1)} | ${s.latencyMs.p95.toFixed(1)} | ${s.latencyMs.max.toFixed(1)} |\n`
report+=`\nЭто IPC + чтение cached DTO, а не latency реального volume/Toggle/WinRT управляющего действия. Точный COM heap/refcount profile не выполнялся; серия не доказывает отсутствие любых утечек.\n\n## Release / NSIS\n\nФинальная сборка: output/stage-7/release-scoped-build.log; тесты: rust-tests-final.log (13/13), frontend-tests-final.log (11/11), typecheck-final.log.\nУстановщик: src-tauri/target/release/bundle/nsis/LUNÉA_0.1.0_x64-setup.exe — **${installer.bytes.toLocaleString('en-US')} bytes**.\nSHA-256: \`${installer.sha256}\`. Версия 0.1.0 сохранена, содержимое — этап 7. Установка автоматически не выполнялась.\nCode Integrity 4551 в финальных проверках не повторилась; защита Windows не менялась.\n`
await writeFile('output/stage-7/metrics-report.md',report)
console.log('Wrote output/stage-7/metrics-report.md')


