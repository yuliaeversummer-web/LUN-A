use std::{sync::{Arc,atomic::{AtomicBool,Ordering}},time::{Instant,Duration}};
use windows::core::Interface;
use windows_future::{IAsyncInfo,AsyncStatus};
use super::errors::{Result,MediaError};
#[derive(Clone)] pub struct Signals {
 pub sessions: Arc<AtomicBool>, pub metadata: Arc<AtomicBool>, pub playback: Arc<AtomicBool>, pub timeline: Arc<AtomicBool>,
}
impl Signals {
 pub fn new() -> Self {Self {sessions:Arc::new(AtomicBool::new(true)),metadata:Arc::new(AtomicBool::new(true)),playback:Arc::new(AtomicBool::new(true)),timeline:Arc::new(AtomicBool::new(true))}}
 pub fn mark(flag: &AtomicBool) {flag.store(true,Ordering::Release);}
 pub fn take(flag: &AtomicBool) -> bool {flag.swap(false,Ordering::AcqRel)}
}
// Only this WinRT worker waits; polling IAsyncInfo allows bounded cancellation
// during shutdown without blocking Tauri's UI thread or using .join() forever.
pub fn wait<O:Interface,T>(operation:&O,stop:&AtomicBool,finish:impl FnOnce()->windows::core::Result<T>) -> Result<T> {
 let info=operation.cast::<IAsyncInfo>()?; let until=Instant::now()+Duration::from_secs(3);
 while info.Status()?==AsyncStatus::Started {
  if stop.load(Ordering::Acquire) || Instant::now()>=until {let _=info.Cancel();return Err(MediaError::new("timeout","Медиаплеер не ответил вовремя. Повторите действие."));}
  std::thread::sleep(Duration::from_millis(15));
 }
 Ok(finish()?)
}
