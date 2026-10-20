import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { synchronizeService } from '../services/serviceSync'
import type { MediaSnapshot } from '../types/media'

export const useMediaStore = defineStore('media', () => {
 const snapshot = ref<MediaSnapshot>({ revision: -1, status: 'loading', reason: 'initializing', message: null, warning: null, selectedId: null, selectedSource: null, selectionMode: 'auto', preferredSource: null, candidates: [], track: null, playback: 'unavailable', capabilities: {play:false,pause:false,toggle:false,next:false,previous:false,seek:false}, timeline:null, reaction:null, commandError:null, persistenceError:null })
 const artwork=ref(''), error=ref(''), busy=ref(false)
 const reaction=ref<{sequence:number;kind:'next'|'error'}|null>(null)
 let reactionSequence=0
 let initialized=false, disposed=false, artworkGeneration=0
 const available=computed(()=>!!snapshot.value.selectedId && ['ready','paused'].includes(snapshot.value.status))
 const playing=computed(()=>available.value && snapshot.value.playback==='playing')
 const canToggle=computed(()=>available.value && (snapshot.value.capabilities.toggle || (playing.value?snapshot.value.capabilities.pause:snapshot.value.capabilities.play)))
 const canSeek=computed(()=>available.value && snapshot.value.capabilities.seek && !!snapshot.value.timeline && snapshot.value.timeline.maxSeekMs>snapshot.value.timeline.minSeekMs)
 function apply(next:MediaSnapshot, live=true) {
  if(disposed||next.revision<snapshot.value.revision)return
  const old=snapshot.value.track?.artworkId, current=next.track?.artworkId
  if(live && next.reaction && snapshot.value.revision>=0 && next.reaction.sequence!==snapshot.value.reaction?.sequence) reaction.value={sequence:++reactionSequence,kind:next.reaction.kind}
  if(snapshot.value.commandError&&!next.commandError)error.value=''
  snapshot.value=next
  if(old!==current){
   artwork.value='';const generation=++artworkGeneration
   if(current)void invoke<string>('get_media_artwork',{artworkId:current}).then(data=>{
    if(!disposed&&generation===artworkGeneration&&snapshot.value.track?.artworkId===current)artwork.value=data
   }).catch(()=>{ /* An unavailable or obsolete cover stays on the placeholder. */ })
  }
 }
 function message(cause:unknown){return cause&&typeof cause==='object'&&'message' in cause?String(cause.message):String(cause)}
 async function request(name:string,args?:Record<string,unknown>){
  if(!isTauri()||busy.value)return false
  error.value='';busy.value=true
  try{apply(await invoke<MediaSnapshot>(name,args));return true}
  catch(cause){error.value=message(cause);return false}
  finally{busy.value=false}
 }
 function command(name:'media_play'|'media_pause'|'media_toggle_playback'|'media_next'|'media_previous'){
  if(!available.value||!snapshot.value.selectedId)return Promise.resolve(false)
  return request(name,{sessionId:snapshot.value.selectedId})
 }
 function seek(positionMs:number){
  if(!canSeek.value||!snapshot.value.track)return Promise.resolve(false)
  return request('media_seek',{sessionId:snapshot.value.selectedId,trackToken:snapshot.value.track.token,positionMs})
 }
 const refresh=()=>request('refresh_media_sessions')
 const select=(sessionId:string|null)=>request('select_media_session',{sessionId})
 async function initialize(){
  if(initialized)return;initialized=true
  if(!isTauri()){snapshot.value.status='unavailable';snapshot.value.reason='platform';snapshot.value.message='Настоящая Яндекс Музыка доступна в приложении LUNÉA для Windows.';return}
  let syncError=false
  synchronizeService<MediaSnapshot>('lunea:media-state','get_media_state',(next,live)=>{
   if(syncError){error.value='';syncError=false}apply(next,live)
  },cause=>{
   syncError=true
   snapshot.value={...snapshot.value,status:'error'};error.value=message(cause)
  },()=>{disposed=true;++artworkGeneration;artwork.value=''})
 }
 return {snapshot,artwork,error,busy,available,playing,canToggle,canSeek,reaction,apply,initialize,command,seek,refresh,select}
})
