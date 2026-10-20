async (page) => {
 const pages=page.context().pages(), main=pages.find(p=>!p.url().includes('/overlay')), overlay=pages.find(p=>p.url().includes('/overlay'));
 const errors=[], results=[];
 for(const p of pages)p.on('pageerror',e=>errors.push(String(e)));
 const original=await main.evaluate(()=>window.__TAURI_INTERNALS__.invoke('get_desktop_state'));
 try {
  await main.evaluate(()=>window.__TAURI_INTERNALS__.invoke('open_main',{settings:false}));
  await main.evaluate(()=>window.__TAURI_INTERNALS__.invoke('configure_overlay',{pinned:true}));
  await main.evaluate(()=>window.__TAURI_INTERNALS__.invoke('toggle_overlay'));
  for(const theme of ['neon-pink','arctic-blue','forest-gold','crimson-night']) {
   await main.evaluate(theme=>window.__TAURI_INTERNALS__.invoke('set_desktop_theme',{theme}),theme);
   await main.waitForFunction(theme=>document.documentElement.dataset.theme===theme,theme,{timeout:5_000});
   await overlay.waitForFunction(theme=>document.documentElement.dataset.theme===theme,theme,{timeout:5_000});
   for(const size of [{width:1100,height:720},{width:850,height:560}]) {
    await main.setViewportSize(size);
    for(const route of ['/','/settings']) {
     await main.evaluate(route=>{location.hash='#'+route},route);
     await main.waitForTimeout(150);
     const check=await main.evaluate(()=>({width:innerWidth,height:innerHeight,overflow:document.documentElement.scrollWidth>innerWidth,mainOverflow:document.querySelector('#main-content').scrollWidth>document.querySelector('#main-content').clientWidth,images:[...document.images].filter(i=>i.complete&&!i.naturalWidth).map(i=>i.getAttribute('src')),theme:document.documentElement.dataset.theme}));
     if(check.overflow||check.mainOverflow||check.images.length)throw Error(JSON.stringify({theme,size,route,...check}));
     results.push({theme,size,route,...check});
     if(route==='/'&&size.width===850)await main.screenshot({path:`output/stage-7/${theme}-main-850.png`});
    }
   }
   await overlay.setViewportSize({width:360,height:440});
   const check=await overlay.evaluate(()=>({overflow:document.documentElement.scrollWidth>innerWidth,theme:document.documentElement.dataset.theme,buttons:[...document.querySelectorAll('button')].map(b=>({label:b.getAttribute('aria-label')||b.textContent.trim(),width:b.getBoundingClientRect().width}))}));
   if(check.overflow)throw Error('Overlay overflow '+theme);
   results.push({overlay:true,theme,...check});
   await overlay.screenshot({path:`output/stage-7/${theme}-overlay.png`});
  }
  await main.emulateMedia({reducedMotion:'reduce'});
  results.push({reducedMotion:await main.evaluate(()=>matchMedia('(prefers-reduced-motion: reduce)').matches)});
  await main.keyboard.press('Tab');
  results.push({focus:await main.evaluate(()=>({tag:document.activeElement?.tagName,label:document.activeElement?.getAttribute('aria-label'),outline:getComputedStyle(document.activeElement).outlineStyle}))});
 } finally {
  await main.evaluate(theme=>window.__TAURI_INTERNALS__.invoke('set_desktop_theme',{theme}),original.preferences.theme||'neon-pink');
  await main.evaluate(p=>window.__TAURI_INTERNALS__.invoke('configure_overlay',{pinned:p.pinned,autoHide:p.autoHide}),original.preferences);
  await overlay.evaluate(()=>window.__TAURI_INTERNALS__.invoke('hide_overlay'));
  await main.emulateMedia({reducedMotion:null});
  for(const p of pages){const cdp=await p.context().newCDPSession(p);await cdp.send('Emulation.clearDeviceMetricsOverride');await cdp.detach();}
  await main.evaluate(()=>{location.hash='#/'});
 }
 if(errors.length)throw Error(errors.join('\n'));
 return {results,errors};
}
