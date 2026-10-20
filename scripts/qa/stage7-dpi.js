async(page)=>{
 const results=[];
 for(const p of page.context().pages()){
  const overlay=p.url().includes('/overlay'),size=overlay?{width:360,height:440}:{width:850,height:560};
  const session=await p.context().newCDPSession(p);
  try{
   for(const scale of [1,1.25,1.5]){
    await session.send('Emulation.setDeviceMetricsOverride',{...size,deviceScaleFactor:scale,mobile:false});
    const check=await p.evaluate(()=>({width:innerWidth,height:innerHeight,dpr:devicePixelRatio,overflow:document.documentElement.scrollWidth>innerWidth,clippedButtons:[...document.querySelectorAll('.overlay-card button')].filter(b=>{const r=b.getBoundingClientRect();return r.right>innerWidth+1||r.bottom>innerHeight+1}).map(b=>b.getAttribute('aria-label')||b.textContent)}));
    if(check.overflow||check.clippedButtons.length)throw Error(JSON.stringify(check));
    results.push({emulationOnly:true,overlay,scale,...check});
    await p.screenshot({path:`output/stage-7/dpi-${scale}-${overlay?'overlay':'main'}.png`});
   }
  }finally{await session.send('Emulation.clearDeviceMetricsOverride');await session.detach();}
 }
 return results;
}
