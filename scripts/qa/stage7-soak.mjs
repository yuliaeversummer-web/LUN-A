// Usage: node stage7-soak.mjs <absolute-playwright-core-index.mjs> <minutes>
// Read-only integrations: only window visibility/pin preferences are changed.
import { appendFile } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'
const { chromium } = await import(pathToFileURL(process.argv[2]).href)
const browser = await chromium.connectOverCDP('http://127.0.0.1:9223')
const pages = browser.contexts()[0].pages()
const main = pages.find(p => !p.url().includes('/overlay'))
const overlay = pages.find(p => p.url().includes('/overlay'))
const invoke = (p, name, args) => p.evaluate(({name,args}) => window.__TAURI_INTERNALS__.invoke(name,args),{name,args})
const saved = await invoke(main,'get_desktop_state')
const started = Date.now(), until = started + Number(process.argv[3] || 30) * 60_000
const output = process.argv[4] || 'output/stage-7/soak-ui.jsonl'
let phase = -1
const errors = []
for (const p of pages) {
 p.on('pageerror',error=>errors.push(String(error)))
 await p.evaluate(()=>{
  const app=document.querySelector('#app').__vue_app__
  const pinia=Object.getOwnPropertySymbols(app._context.provides).map(s=>app._context.provides[s]).find(x=>x?._s instanceof Map)
  window.__stage7Soak={pinia,counts:{audio:0,media:0,discord:0},stops:[]}
  for(const name of ['audio','media','discord'])window.__stage7Soak.stops.push(pinia._s.get(name).$subscribe(()=>window.__stage7Soak.counts[name]++,{flush:'sync',detached:true}))
 })
}
try {
 await invoke(main,'configure_overlay',{pinned:true})
 for(let sample=0;;sample++) {
  const nextPhase = Math.floor(sample / 2) % 4
  if(nextPhase!==phase) {
   phase=nextPhase
   await invoke(overlay,'hide_overlay')
   await invoke(main,'plugin:window|close',{label:'main'})
   if(phase===1)await invoke(main,'open_main',{settings:false})
   if(phase===2)await invoke(main,'toggle_overlay')
  }
  const row={time:new Date().toISOString(),elapsedMs:Date.now()-started,phase,sample,states:{},windows:[],errors:[...errors]}
  for(const name of ['audio','media','discord']) {
   const start=performance.now(),state=await invoke(main,`get_${name}_state`)
   row.states[name]={latencyMs:performance.now()-start,revision:state.revision,status:state.status}
   if(name==='audio')row.states.audio.sessions=state.sessions.length
   if(name==='media')Object.assign(row.states.media,{source:state.selectedSource,trackToken:state.track?.token,artworkId:state.track?.artworkId,position:state.timeline?.positionMs,duration:state.timeline?.durationMs})
   if(name==='discord')Object.assign(row.states.discord,{mute:state.mute,deafen:state.deafen,clientCount:state.clients.filter(c=>c.running).length})
  }
  for(const p of pages)row.windows.push(await p.evaluate(()=>({url:location.href,hidden:document.documentElement.dataset.viewHidden,counts:{...window.__stage7Soak.counts},revisions:Object.fromEntries(['audio','media','discord'].map(name=>[name,window.__stage7Soak.pinia._s.get(name).snapshot.revision]))})))
  await appendFile(output,JSON.stringify(row)+'\n')
  if(Date.now()>=until)break
  await new Promise(resolve=>setTimeout(resolve,Math.min(30_000,until-Date.now())))
 }
} catch(error) {
 await appendFile(output,JSON.stringify({time:new Date().toISOString(),fatal:String(error),elapsedMs:Date.now()-started})+'\n')
 throw error
} finally {
 await invoke(main,'configure_overlay',{pinned:saved.preferences.pinned,autoHide:saved.preferences.autoHide}).catch(()=>{})
 await invoke(overlay,'hide_overlay').catch(()=>{})
 await invoke(main,'open_main',{settings:false}).catch(()=>{})
 for(const p of pages)await p.evaluate(()=>{window.__stage7Soak?.stops.forEach(stop=>stop());delete window.__stage7Soak}).catch(()=>{})
 // Disconnect this QA client; do not close the user's WebView/browser.
 await browser.close()
}
