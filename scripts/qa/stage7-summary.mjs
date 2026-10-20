import { readFile, writeFile } from 'node:fs/promises'
const readLines = async path => (await readFile(path,'utf8')).trim().split(/\r?\n/).filter(Boolean).map(line=>JSON.parse(line))
const metrics = await readLines(process.argv[2] || 'output/stage-7/soak-final-process.jsonl')
const ui = await readLines(process.argv[3] || 'output/stage-7/soak-final-ui.jsonl')
const values = (rows,key) => rows.map(row=>row[key]).filter(Number.isFinite)
const range = (rows,key) => { const v=values(rows,key);return {first:v[0],last:v.at(-1),min:Math.min(...v),max:Math.max(...v)} }
const quantile = (values,q) => {const sorted=[...values].sort((a,b)=>a-b);return sorted[Math.min(sorted.length-1,Math.floor(sorted.length*q))]}
const last = metrics.at(-1), first = metrics[0]
const durationSeconds = (Date.parse(last.time)-Date.parse(first.time))/1000
const uiAt = time => ui.findLast(row=>Date.parse(row.time)<=time)
const hiddenIntervals=[]
let negativeTreeCpuIntervals=0
for(let i=1;i<metrics.length;i++) {
 const a=metrics[i-1],b=metrics[i],start=Date.parse(a.time),end=Date.parse(b.time)
 if(b.treeCpuSeconds<a.treeCpuSeconds)negativeTreeCpuIntervals++
 const before=uiAt(start),after=uiAt(end)
 if(!before||!after||before.elapsedMs<180_000)continue
 if(before.windows.every(w=>w.hidden==='true')&&after.windows.every(w=>w.hidden==='true'))hiddenIntervals.push({seconds:(end-start)/1000,cpu:b.cpuSeconds-a.cpuSeconds,treeCpu:b.treeCpuSeconds-a.treeCpuSeconds})
}
const sum=(rows,key)=>rows.reduce((total,row)=>total+row[key],0)
const hiddenSeconds=sum(hiddenIntervals,'seconds')
const perService={}
for(const name of ['audio','media','discord']) {
 const rows=ui.filter(row=>row.states?.[name])
 const latencies=rows.map(row=>row.states[name].latencyMs)
 perService[name]={latencyMs:{median:quantile(latencies,.5),p95:quantile(latencies,.95),max:Math.max(...latencies)},statuses:rows.reduce((out,row)=>{const state=row.states[name].status;out[state]=(out[state]||0)+1;return out},{})}
}
const hiddenMutations=[],visibleRates=[],visibleLag=[]
for(let i=1;i<ui.length;i++) {
 const previous=ui[i-1],current=ui[i]
 if(!previous.windows||!current.windows)continue
 if(previous.elapsedMs<180_000)continue // Initial UI matrix/emulation is not idle sampling.
 for(const window of current.windows) {
  const old=previous.windows.find(w=>w.url===window.url)
  if(!old)continue
  const mutations=window.counts.media-old.counts.media
  if(window.hidden==='true'&&old.hidden==='true')hiddenMutations.push(mutations)
  if(window.hidden==='false'&&old.hidden==='false')visibleRates.push(mutations/((current.elapsedMs-previous.elapsedMs)/1000))
  if(window.hidden==='false'&&current.states?.media)visibleLag.push(Math.max(0,current.states.media.revision-window.revisions.media))
 }
}
const summary={
 samples:metrics.length,uiSamples:ui.length,durationSeconds,uiDurationSeconds:ui.at(-1).elapsedMs/1000,
 processId:first.pid,oneCoreCpuPercent:100*(last.cpuSeconds-first.cpuSeconds)/durationSeconds,
 treeOneCoreCpuPercent:100*(last.treeCpuSeconds-first.treeCpuSeconds)/durationSeconds,
 root:Object.fromEntries(['workingSet','privateBytes','handles','threads'].map(key=>[key,range(metrics,key)])),
 tree:Object.fromEntries(['treeWorkingSet','treePrivateBytes','treeHandles','treeThreads','processCount'].map(key=>[key,range(metrics,key)])),
 workersAlwaysOne:metrics.every(row=>['lunea-core-audio','lunea-gsmtc','lunea-discord'].every(name=>row.workers?.[name]===1)),
 hidden:{seconds:hiddenSeconds,oneCoreCpuPercent:hiddenSeconds?100*sum(hiddenIntervals,'cpu')/hiddenSeconds:null,treeOneCoreCpuPercent:hiddenSeconds?100*sum(hiddenIntervals,'treeCpu')/hiddenSeconds:null,mutationIntervals:hiddenMutations.length,maxMediaMutations:Math.max(...hiddenMutations)},
 visibleMediaUpdatesPerSecond:{median:quantile(visibleRates,.5),max:Math.max(...visibleRates)},
 maxVisibleMediaRevisionLag:Math.max(...visibleLag),negativeTreeCpuIntervals,perService,
 pageErrors:[...new Set(ui.flatMap(row=>row.errors||[]))],fatal:ui.filter(row=>row.fatal),
 discordRemainedUnknown:ui.every(row=>!row.states?.discord||(row.states.discord.mute===null&&row.states.discord.deafen===null)),
 sources:[...new Set(ui.map(row=>row.states?.media?.source).filter(Boolean))],
 trackTokens:[...new Set(ui.map(row=>row.states?.media?.trackToken).filter(v=>v!==undefined))],
}
await writeFile(process.argv[4]||'output/stage-7/soak-summary.json',JSON.stringify(summary,null,2))
console.log(JSON.stringify({samples:summary.samples,seconds:summary.durationSeconds,workersAlwaysOne:summary.workersAlwaysOne,pageErrors:summary.pageErrors.length,fatal:summary.fatal.length,hidden:summary.hidden}))
