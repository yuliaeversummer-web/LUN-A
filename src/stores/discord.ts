import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { synchronizeService } from '../services/serviceSync'
import type { DiscordPreferences, DiscordSnapshot } from '../types/discord'
export const useDiscordStore=defineStore('discord',()=>{
 const snapshot=ref<DiscordSnapshot>({revision:-1,configurationRevision:0,preferences:{client:'stable',enabled:false,confirmed:false,mute:'',deafen:''},clients:[],status:'loading',reason:'initializing',message:'Обнаружение Discord…',canSend:false,mode:'hotkeys',mute:null,deafen:null,audioSessions:[],lastAttempt:null,persistenceError:null})
 const busy=ref(false),error=ref(''),reaction=ref(0)
 let initialized=false,disposed=false,lastLocal=0
 const canSend=computed(()=>snapshot.value.canSend&&!busy.value)
 const running=computed(()=>snapshot.value.clients.some(c=>c.variant===snapshot.value.preferences.client&&c.running))
 function apply(next:DiscordSnapshot,live=true){if(disposed||next.revision<snapshot.value.revision)return;if(live&&snapshot.value.revision>=0&&next.lastAttempt?.status==='not_sent'&&next.lastAttempt.sequence!==snapshot.value.lastAttempt?.sequence)reaction.value++;snapshot.value=next}
 function message(cause:unknown){return cause&&typeof cause==='object'&&'message'in cause?String(cause.message):String(cause)}
 async function request(name:string,args?:Record<string,unknown>){if(!isTauri()||busy.value)return false;error.value='';busy.value=true;try{apply(await invoke<DiscordSnapshot>(name,args));return true}catch(cause){error.value=message(cause);return false}finally{busy.value=false}}
 async function toggle(action:'mute'|'deafen'){
  if(!canSend.value||performance.now()-lastLocal<1200)return false;lastLocal=performance.now()
  return request(`discord_toggle_${action}`,{client:snapshot.value.preferences.client,configurationRevision:snapshot.value.configurationRevision})
 }
 const configure=(preferences:DiscordPreferences)=>request('configure_discord',{preferences})
 const refresh=()=>request('refresh_discord')
 async function initialize(){if(initialized)return;initialized=true;if(!isTauri()){snapshot.value.status='unavailable';snapshot.value.message='Интеграция Discord доступна в LUNÉA для Windows.';return}
  let syncError=false
  synchronizeService<DiscordSnapshot>('lunea:discord-state','get_discord_state',(next,live)=>{if(syncError){error.value='';syncError=false}apply(next,live)},cause=>{syncError=true;snapshot.value={...snapshot.value,status:'error',canSend:false};error.value=message(cause)},()=>{disposed=true})
 }
 return {snapshot,busy,error,reaction,canSend,running,apply,toggle,configure,refresh,initialize}
})
