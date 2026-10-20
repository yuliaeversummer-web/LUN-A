//! Explicit QA tool, never started by LUNÉA or bundled. Only the two user-confirmed test chords.
#![allow(dead_code)]
#[cfg(windows)]
#[path="../src/discord/errors.rs"]mod errors;
#[cfg(windows)]
#[path="../src/discord/hotkeys.rs"]mod hotkeys;
#[cfg(windows)]
#[path="../src/discord/detection.rs"]mod detection;
#[cfg(windows)]
#[path="../src/discord/input.rs"]mod input;
#[cfg(windows)]#[derive(serde::Serialize)]pub struct Client{pub variant:String,pub installed:bool,pub running:bool,pub pids:Vec<u32>,pub privilege_compatible:bool}
#[cfg(windows)]#[derive(Default)]pub struct Preferences{pub client:String,pub enabled:bool,pub confirmed:bool,pub mute:String,pub deafen:String}
#[cfg(windows)]fn main(){
 let action=std::env::args().nth(1);let clients=detection::scan();
 if action.is_none(){println!("{}",serde_json::to_string(&clients).unwrap());return;}
 let clients=match clients{Ok(c)=>c,Err(e)=>{eprintln!("{}",e.message);std::process::exit(1);}};
 if clients.iter().filter(|c|c.running).count()!=1||!clients.iter().any(|c|c.variant=="stable"&&c.running&&c.privilege_compatible){eprintln!("Requires only verified Stable at compatible integrity");std::process::exit(1);}
 let key=match action.as_deref(){Some("mute")=>"Ctrl+Alt+Shift+F9",Some("deafen")=>"Ctrl+Alt+Shift+F10",_=>{eprintln!("Only mute/deafen test actions");std::process::exit(1);}};
 let result=input::send(&hotkeys::parse(key).unwrap());println!("{}",serde_json::json!({"action":action,"result":result,"state":"unknown"}));
}
#[cfg(not(windows))]fn main(){}
