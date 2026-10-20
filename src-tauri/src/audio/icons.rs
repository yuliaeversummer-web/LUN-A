use base64::Engine;
use windows::{core::{PCWSTR, PWSTR}, Win32::{Foundation::CloseHandle, System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_NAME_WIN32}, Graphics::Gdi::*, UI::{Shell::ExtractIconExW, WindowsAndMessaging::{GetIconInfo, DestroyIcon, HICON, ICONINFO}}}};

pub fn process_path(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = vec![0u16; 32768]; let mut length = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut length);
        let _ = CloseHandle(process);
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..length as usize]))
    }
}
pub fn app_icon(path: &str) -> Option<String> {
    // Only extract a tiny local executable resource. Never expose filesystem
    // paths to the WebView or capture sound. All GDI handles are paired below.
    unsafe {
        let path: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        let mut icon = HICON::default();
        if ExtractIconExW(PCWSTR(path.as_ptr()), 0, None, Some(&mut icon), 1) == 0 { return None; }
        let result = icon_png(icon);
        let _ = DestroyIcon(icon);
        result
    }
}
unsafe fn icon_png(icon: HICON) -> Option<String> {
    unsafe {
        let mut info = ICONINFO::default();
        GetIconInfo(icon, &mut info).ok()?;
        let dc = CreateCompatibleDC(None);
        let result = (|| {
            let mut bitmap = BITMAP::default();
            if GetObjectW(info.hbmColor.into(), std::mem::size_of::<BITMAP>() as i32, Some((&mut bitmap as *mut BITMAP).cast())) == 0 { return None; }
            let (width, height) = (bitmap.bmWidth, bitmap.bmHeight);
            if width <= 0 || height <= 0 || width > 64 || height > 64 { return None; }
            let mut header = BITMAPINFO::default();
            header.bmiHeader = BITMAPINFOHEADER { biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32, biWidth: width, biHeight: -height, biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0, ..Default::default() };
            let mut pixels = vec![0u8; (width * height * 4) as usize];
            if GetDIBits(dc, info.hbmColor, 0, height as u32, Some(pixels.as_mut_ptr().cast()), &mut header, DIB_RGB_COLORS) == 0 { return None; }
            // Legacy monochrome/zero-alpha icons use a generic UI fallback.
            if pixels.chunks_exact(4).all(|pixel| pixel[3] == 0) { return None; }
            for pixel in pixels.chunks_exact_mut(4) { pixel.swap(0, 2); }
            let mut png = Vec::new();
            { let mut encoder = png::Encoder::new(&mut png, width as u32, height as u32);
              encoder.set_color(png::ColorType::Rgba); encoder.set_depth(png::BitDepth::Eight);
              encoder.write_header().ok()?.write_image_data(&pixels).ok()?; }
            Some(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(png)))
        })();
        let _ = DeleteDC(dc); let _ = DeleteObject(info.hbmColor.into()); let _ = DeleteObject(info.hbmMask.into());
        result
    }
}
