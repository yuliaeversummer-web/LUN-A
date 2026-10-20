use super::{events::Signals,policy,Candidate};
use windows::{core::{Interface,IUnknown},Foundation::TypedEventHandler,Media::Control::*};
pub type Session=GlobalSystemMediaTransportControlsSession;
pub type Manager=GlobalSystemMediaTransportControlsSessionManager;
pub fn same(a:&Session,b:&Session)->bool {match(a.cast::<IUnknown>(),b.cast::<IUnknown>()){(Ok(a),Ok(b))=>a==b,_=>false}}
pub struct SessionEntry {pub session:Session,pub dto:Candidate,tokens:Vec<(u8,i64)>}
impl SessionEntry {
 pub fn new(session:Session,id:String,signals:&Signals)->windows::core::Result<Self> {
  let source=session.SourceAppUserModelId()?.to_string(); let mut tokens=Vec::new();
  // Subscribe only to eligible desktop sessions. Other players are read-only
  // discovery IDs, and never supply metadata or command recipients.
  if policy::eligible(&source) {
   let flag=signals.metadata.clone();
   if let Ok(token)=session.MediaPropertiesChanged(&TypedEventHandler::new(move|_,_|{Signals::mark(&flag);Ok(())})) {tokens.push((0,token));}
   let flag=signals.playback.clone();
   if let Ok(token)=session.PlaybackInfoChanged(&TypedEventHandler::new(move|_,_|{Signals::mark(&flag);Ok(())})) {tokens.push((1,token));}
   let flag=signals.timeline.clone();
   if let Ok(token)=session.TimelinePropertiesChanged(&TypedEventHandler::new(move|_,_|{Signals::mark(&flag);Ok(())})) {tokens.push((2,token));}
  }
  let dto=Candidate {id,eligible:policy::eligible(&source),name:if policy::eligible(&source){"Яндекс Музыка · десктоп"}else{"Другой источник · управление запрещено"}.into(),source_id:source};
  Ok(Self {session,dto,tokens})
 }
 pub fn fully_observed(&self)->bool {!self.dto.eligible || self.tokens.len()==3}
}
impl Drop for SessionEntry {
 fn drop(&mut self){for(kind,token)in self.tokens.drain(..){let _=match kind{0=>self.session.RemoveMediaPropertiesChanged(token),1=>self.session.RemovePlaybackInfoChanged(token),_=>self.session.RemoveTimelinePropertiesChanged(token)};}}
}
pub struct ManagerEvents {manager:Manager,tokens:Vec<(u8,i64)>}
impl ManagerEvents {
 pub fn new(manager:&Manager,signals:&Signals)->Self {
  let mut tokens=Vec::new(); let flag=signals.sessions.clone();
  if let Ok(token)=manager.SessionsChanged(&TypedEventHandler::new(move|_,_|{Signals::mark(&flag);Ok(())})){tokens.push((0,token));}
  let flag=signals.sessions.clone();
  if let Ok(token)=manager.CurrentSessionChanged(&TypedEventHandler::new(move|_,_|{Signals::mark(&flag);Ok(())})){tokens.push((1,token));}
  Self {manager:manager.clone(),tokens}
 }
 pub fn fully_observed(&self)->bool {self.tokens.len()==2}
}
impl Drop for ManagerEvents {fn drop(&mut self){for(kind,token)in self.tokens.drain(..){let _=if kind==0{self.manager.RemoveSessionsChanged(token)}else{self.manager.RemoveCurrentSessionChanged(token)};}}}
