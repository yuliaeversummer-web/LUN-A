async(page)=>{
 const pages=page.context().pages(),main=pages.find(p=>!p.url().includes('/overlay')),overlay=pages.find(p=>p.url().includes('/overlay'));
 const invoke=(p,name,args)=>p.evaluate(({name,args})=>window.__TAURI_INTERNALS__.invoke(name,args),{name,args});
 const saved=await invoke(main,'get_desktop_state'),result=[];let backendStart=0;
 const reset=async()=>{backendStart=(await invoke(main,'get_media_state')).revision;for(const p of pages)await p.evaluate(()=>{window.__hiddenQA.count=0;window.__hiddenQA.start=Date.now()})};
 const capture=async name=>{const backendDelta=(await invoke(main,'get_media_state')).revision-backendStart;for(const p of pages)result.push({...await p.evaluate(name=>({phase:name,url:location.href,seconds:(Date.now()-window.__hiddenQA.start)/1000,snapshotUpdates:window.__hiddenQA.count,hidden:document.documentElement.dataset.viewHidden}),name),backendDelta})};
 try{
  for(const p of pages)await p.evaluate(()=>{const app=document.querySelector('#app').__vue_app__;const pinia=Object.getOwnPropertySymbols(app._context.provides).map(s=>app._context.provides[s]).find(x=>x?._s instanceof Map);const media=pinia._s.get('media');window.__hiddenQA={count:0,last:media.snapshot.revision,start:Date.now(),stop:media.$subscribe((_mutation,state)=>{if(state.snapshot.revision!==window.__hiddenQA.last){window.__hiddenQA.last=state.snapshot.revision;window.__hiddenQA.count++}},{flush:'sync',detached:true})};});
  await invoke(main,'configure_overlay',{pinned:true});
  await invoke(overlay,'hide_overlay');await invoke(main,'plugin:window|close',{label:'main'});await main.waitForTimeout(250);await reset();
  await invoke(main,'open_main',{settings:false});await main.waitForTimeout(4000);await capture('main-visible');
  await invoke(main,'plugin:window|close',{label:'main'});await invoke(main,'toggle_overlay');await main.waitForTimeout(250);await reset();await main.waitForTimeout(4000);await capture('overlay-visible');
  await invoke(overlay,'hide_overlay');await main.waitForTimeout(250);await reset();await main.waitForTimeout(4000);await capture('both-hidden');
  const wrong=result.filter(r=>(r.phase==='both-hidden'||r.phase==='main-visible'&&r.url.includes('/overlay')||r.phase==='overlay-visible'&&!r.url.includes('/overlay'))&&r.snapshotUpdates!==0);
  if(wrong.length)throw Error(JSON.stringify(wrong));
  const frozen=result.filter(r=>r.phase!=='both-hidden'&&r.hidden==='false'&&r.backendDelta>0&&r.snapshotUpdates===0);
  if(frozen.length)throw Error('Visible view stopped synchronizing: '+JSON.stringify(frozen));
  return result;
 }finally{
  for(const p of pages)await p.evaluate(()=>{window.__hiddenQA?.stop();delete window.__hiddenQA});
  await invoke(main,'configure_overlay',{pinned:saved.preferences.pinned,autoHide:saved.preferences.autoHide});await invoke(overlay,'hide_overlay');await invoke(main,'open_main',{settings:false});
 }
}
