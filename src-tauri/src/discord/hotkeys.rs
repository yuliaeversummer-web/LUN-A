use super::{errors::{DiscordError,Result},Preferences};
use std::str::FromStr;
#[derive(Clone,Debug,PartialEq)]pub struct Chord {pub canonical:String,pub key:u16}
pub fn parse(value:&str)->Result<Chord>{
 if value.len()>64{return Err(DiscordError::new("invalid_hotkey","Сочетание слишком длинное."));}
 let parts:Vec<_>=value.split('+').map(|s|s.trim().to_ascii_uppercase()).collect();
 if parts.len()!=4||parts[..3].iter().collect::<std::collections::HashSet<_>>().len()!=3||!parts[..3].iter().all(|s|["CTRL","ALT","SHIFT"].contains(&s.as_str())){return Err(DiscordError::new("invalid_hotkey","Допустимы Ctrl+Alt+Shift и одна клавиша F6–F11. Текст, Win и одиночные клавиши запрещены."));}
 let number=parts[3].strip_prefix('F').and_then(|s|s.parse::<u16>().ok()).filter(|n|(6..=11).contains(n)&&parts[3]==format!("F{n}")).ok_or_else(||DiscordError::new("invalid_hotkey","Выберите функциональную клавишу F6–F11 с Ctrl+Alt+Shift."))?;
 Ok(Chord {canonical:format!("Ctrl+Alt+Shift+F{number}"),key:0x70+number-1})
}
pub fn same(a:&str,b:&str)->bool{
 use tauri_plugin_global_shortcut::Shortcut;
 match(Shortcut::from_str(a),Shortcut::from_str(b)){(Ok(a),Ok(b))=>a==b,_=>false}
}
pub fn validate(p:&Preferences,overlay:&str)->Result<()> {
 if !["stable","ptb","canary"].contains(&p.client.as_str()){return Err(DiscordError::new("invalid_client","Выберите Stable, PTB или Canary."));}
 let mute=if p.mute.is_empty(){None}else{Some(parse(&p.mute)?)};let deafen=if p.deafen.is_empty(){None}else{Some(parse(&p.deafen)?)};
 if mute.as_ref().zip(deafen.as_ref()).is_some_and(|(a,b)|a==b){return Err(DiscordError::new("duplicate_hotkey","Mute и Deafen должны иметь разные сочетания."));}
 for key in [mute,deafen].into_iter().flatten(){if same(&key.canonical,overlay){return Err(DiscordError::new("overlay_conflict","Сочетание совпадает с горячей клавишей оверлея. Настройки не изменены."));}}
 if p.enabled&&(!p.confirmed||p.mute.is_empty()||p.deafen.is_empty()){return Err(DiscordError::new("not_confirmed","Сначала назначьте оба сочетания в выбранном Discord и подтвердите это в настройках LUNÉA."));}Ok(())
}
#[cfg(test)]mod tests{use super::*;
 #[test]fn accepts_only_bounded_chords(){assert_eq!(parse("Shift + Ctrl + Alt + F9").unwrap().canonical,"Ctrl+Alt+Shift+F9");for key in ["Ctrl+D","Win+L","Ctrl+Alt+Del","Ctrl+Alt+Shift+F4","Ctrl+Alt+Alt+F9","Ctrl+Alt+Shift+F12","Ctrl+Alt+Shift+F13","Ctrl+Alt+Shift+F09","a;shutdown","Ctrl+Alt+Shift+F9+F10"]{assert!(parse(key).is_err(),"{key}");}}
 #[test]fn blocks_duplicate_and_overlay_chords(){let mut p=Preferences{enabled:true,confirmed:true,mute:"Ctrl+Alt+Shift+F9".into(),deafen:"Ctrl+Alt+Shift+F10".into(),..Default::default()};assert!(validate(&p,"Ctrl+D").is_ok());assert!(validate(&p,"Alt+Shift+Ctrl+F9").is_err());p.deafen=p.mute.clone();assert!(validate(&p,"Ctrl+D").is_err());p.enabled=false;p.client="browser".into();assert!(validate(&p,"Ctrl+D").is_err());}
}
