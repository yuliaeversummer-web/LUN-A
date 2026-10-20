use serde::Serialize;
#[derive(Clone,Debug,Serialize,PartialEq)]
pub struct DiscordError {pub code:String,pub message:String}
impl DiscordError {pub fn new(code:&str,message:&str)->Self{Self{code:code.into(),message:message.into()}}}
pub type Result<T> = std::result::Result<T,DiscordError>;

