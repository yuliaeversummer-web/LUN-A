async (page) => {
 const pages=page.context().pages(),main=pages.find(p=>!p.url().includes('/overlay'));
 const result=[];
 try {
  for(const p of pages)await p.evaluate(()=>{
   const app=document.querySelector('#app').__vue_app__;
   const pinia=Object.getOwnPropertySymbols(app._context.provides).map(s=>app._context.provides[s]).find(x=>x?._s instanceof Map);
   const saved=Object.fromEntries(['audio','media','discord'].map(key=>[key,JSON.parse(JSON.stringify(pinia._s.get(key).snapshot))]));
   window.__stage7Stress={pinia,saved};
   const audio=pinia._s.get('audio'),media=pinia._s.get('media');
   audio.snapshot={...audio.snapshot,revision:9000000,sessions:Array.from({length:300},(_,i)=>({...saved.audio.sessions[0],id:`synthetic-layout-${i}`,name:'Очень длинное название приложения '.repeat(12),deviceName:'DeviceWithoutSpaces'.repeat(25)}))};
   if(media.snapshot.track)media.snapshot={...media.snapshot,revision:9000000,track:{...media.snapshot.track,title:'Композиция с длинным названием '.repeat(20),artist:'ArtistWithoutSpaces'.repeat(20),album:'Album'.repeat(100)}};
  });
  await main.setViewportSize({width:850,height:560});
  for(const p of pages){
   if(p!==main)await p.setViewportSize({width:360,height:440});
   await p.waitForTimeout(200);
   result.push(await p.evaluate(()=>({synthetic:true,url:location.href,width:innerWidth,overflow:document.documentElement.scrollWidth>innerWidth,sessionCount:document.querySelectorAll('.audio-session').length,scrollContainers:[...document.querySelectorAll('.audio-sessions,.media-card,.discord-card')].map(e=>({className:e.className,client:e.clientWidth,scroll:e.scrollWidth}))})));
   await p.screenshot({path:`output/stage-7/stress-${p===main?'main':'overlay'}.png`});
  }
  // Error/unavailable fixtures are layout only, never actual backend failures.
  await main.evaluate(()=>{const {pinia}=window.__stage7Stress;pinia._s.get('audio').snapshot={...pinia._s.get('audio').snapshot,status:'unavailable',endpoint:null,sessions:[],outputs:[],inputs:[],message:'Тест отсутствия устройства'};pinia._s.get('media').snapshot={...pinia._s.get('media').snapshot,status:'error',selectedId:null,track:null,timeline:null,message:'Тест временной ошибки GSMTC'};pinia._s.get('discord').snapshot={...pinia._s.get('discord').snapshot,status:'unavailable',canSend:false,clients:[],audioSessions:[],message:'Тест отсутствия Discord'};});
  await main.locator('.native-mixer').scrollIntoViewIfNeeded();
  await main.screenshot({path:'output/stage-7/stress-unavailable.png'});
  result.push(await main.evaluate(()=>({fixture:'unavailable',horizontal:document.documentElement.scrollWidth>innerWidth,disabledMaster:document.querySelector('input[aria-label="Системная громкость"]').disabled})));
 } finally {
  for(const p of pages){
   await p.evaluate(()=>{const state=window.__stage7Stress;if(!state)return;for(const key of ['audio','media','discord'])state.pinia._s.get(key).snapshot=state.saved[key];delete window.__stage7Stress;});
   const session=await p.context().newCDPSession(p);await session.send('Emulation.clearDeviceMetricsOverride');await session.detach();
  }
 }
 if(result.some(row=>row.overflow||row.horizontal||row.scrollContainers?.some(e=>e.scroll>e.client+1)))throw Error(JSON.stringify(result));
 return result;
}
