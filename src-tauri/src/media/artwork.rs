use std::sync::atomic::AtomicBool;
use base64::Engine;
use windows::{Graphics::Imaging::BitmapDecoder,Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,Storage::Streams::{DataReader,IRandomAccessStreamWithContentType}};
use super::{events::wait,errors::{Result,MediaError}};
const MAX_BYTES:u64=512*1024;
struct Stream(IRandomAccessStreamWithContentType);
impl Drop for Stream{fn drop(&mut self){let _=self.0.Close();}}
pub fn read(properties:&GlobalSystemMediaTransportControlsSessionMediaProperties,stop:&AtomicBool)->Result<String>{
 let reference=properties.Thumbnail()?; let operation=reference.OpenReadAsync()?;
 let stream=Stream(wait(&operation,stop,||operation.GetResults())?);
 let size=stream.0.Size()?;
 if size==0 || size>MAX_BYTES {return Err(MediaError::new("artwork_size","Обложка отсутствует или превышает лимит 512 КиБ."));}
 let mime=stream.0.ContentType()?.to_string().to_ascii_lowercase();
 if !["image/png","image/jpeg","image/webp"].contains(&mime.as_str()){return Err(MediaError::new("artwork_type","Формат обложки Windows не поддерживается."));}
 let decoder=BitmapDecoder::CreateAsync(&stream.0)?;
 let decoder=wait(&decoder,stop,||decoder.GetResults())?;
 let (width,height)=(decoder.PixelWidth()?,decoder.PixelHeight()?);
 if width==0 || height==0 || width>2048 || height>2048 {return Err(MediaError::new("artwork_dimensions","Размер обложки превышает 2048×2048."));}
 drop(decoder);
 let input=stream.0.GetInputStreamAt(0)?; let reader=DataReader::CreateDataReader(&input)?;
 let result=(||{let load=reader.LoadAsync(size as u32)?; let count=wait(&load,stop,||load.GetResults())?;
  if count!=size as u32{return Err(MediaError::new("artwork_read","Windows вернул неполную обложку."));}
  let mut bytes=vec![0u8;count as usize];reader.ReadBytes(&mut bytes)?;
  Ok(format!("data:{mime};base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes)))})();
 let _=reader.Close();result
}
