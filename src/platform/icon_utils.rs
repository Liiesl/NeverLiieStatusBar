#[cfg(target_os = "windows")]
mod inner {
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, GetDIBits, GetObjectW, SelectObject, BITMAP, BITMAPINFO,
        BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetIconInfoExW, HICON, ICONINFOEXW};

    /// Converts an HICON handle to RGBA pixel data.
    /// Returns (rgba_bytes, width, height) on success.
    pub fn hicon_to_rgba(hicon: isize) -> Option<(Vec<u8>, u32, u32)> {
        unsafe {
            let hicon = HICON(hicon as *mut _);

            let mut icon_info = ICONINFOEXW {
                cbSize: std::mem::size_of::<ICONINFOEXW>() as u32,
                ..Default::default()
            };

            if !GetIconInfoExW(hicon, &mut icon_info).as_bool() {
                return None;
            }

            let hbm_color = icon_info.hbmColor;
            let hbm_mask = icon_info.hbmMask;

            let result = if !hbm_color.is_invalid() {
                hbitmap_to_rgba(hbm_color)
            } else if !hbm_mask.is_invalid() {
                hbitmap_to_rgba(hbm_mask)
            } else {
                None
            };

            if !hbm_color.is_invalid() {
                let _ = windows::Win32::Graphics::Gdi::DeleteObject(hbm_color.into());
            }
            if !hbm_mask.is_invalid() {
                let _ = windows::Win32::Graphics::Gdi::DeleteObject(hbm_mask.into());
            }

            result
        }
    }

    unsafe fn hbitmap_to_rgba(hbitmap: windows::Win32::Graphics::Gdi::HBITMAP) -> Option<(Vec<u8>, u32, u32)> {
        let mut bitmap = BITMAP::default();
        unsafe {
            if GetObjectW(
                hbitmap.into(),
                std::mem::size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as *mut _),
            ) == 0
            {
                return None;
            }
        }

        let width = bitmap.bmWidth;
        let height = bitmap.bmHeight.abs();

        unsafe {
            let hdc_screen = CreateCompatibleDC(None);
            let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
            let hbm_old = SelectObject(hdc_mem, hbitmap.into());

            let mut bmp_info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -(height),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut buffer = vec![0u8; (width * height * 4) as usize];

            let result = GetDIBits(
                hdc_mem,
                hbitmap,
                0,
                height as u32,
                Some(buffer.as_mut_ptr() as *mut _),
                &mut bmp_info,
                DIB_RGB_COLORS,
            );

            SelectObject(hdc_mem, hbm_old);
            let _ = DeleteDC(hdc_mem);
            let _ = DeleteDC(hdc_screen);

            if result == 0 {
                return None;
            }

            fix_alpha_channel(buffer.as_mut_slice());
            bgra_to_rgba(buffer.as_mut_slice());

            Some((buffer, width as u32, height as u32))
        }
    }

    fn fix_alpha_channel(buffer: &mut [u8]) {
        let pixels = buffer.len() / 4;
        let mut has_non_zero_alpha = false;
        let mut has_varying_alpha = false;

        for i in 0..pixels {
            let alpha = buffer[i * 4 + 3];
            if alpha > 0 {
                has_non_zero_alpha = true;
            }
            if alpha > 0 && alpha < 255 {
                has_varying_alpha = true;
                break;
            }
        }

        if !has_non_zero_alpha {
            for i in 0..pixels {
                buffer[i * 4 + 3] = 255;
            }
        } else if has_varying_alpha {
            for i in 0..pixels {
                let alpha = buffer[i * 4 + 3];
                if alpha > 0 && alpha < 255 {
                    let alpha_f = alpha as f32 / 255.0;
                    let b = ((buffer[i * 4] as f32 / alpha_f).min(255.0)) as u8;
                    let g = ((buffer[i * 4 + 1] as f32 / alpha_f).min(255.0)) as u8;
                    let r = ((buffer[i * 4 + 2] as f32 / alpha_f).min(255.0)) as u8;
                    buffer[i * 4] = b;
                    buffer[i * 4 + 1] = g;
                    buffer[i * 4 + 2] = r;
                }
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn bgra_to_rgba(data: &mut [u8]) {
        use std::arch::x86_64::{__m128i, _mm_loadu_si128, _mm_setr_epi8, _mm_shuffle_epi8, _mm_storeu_si128};

        unsafe {
            let mask = _mm_setr_epi8(2, 1, 0, 3, 6, 5, 4, 7, 10, 9, 8, 11, 14, 13, 12, 15);
            for chunk in data.chunks_exact_mut(16) {
                let mut vector = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
                vector = _mm_shuffle_epi8(vector, mask);
                _mm_storeu_si128(chunk.as_mut_ptr() as *mut __m128i, vector);
            }
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn bgra_to_rgba(data: &mut [u8]) {
        for chunk in data.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod inner {
    pub fn hicon_to_rgba(_hicon: isize) -> Option<(Vec<u8>, u32, u32)> {
        None
    }
}

pub use inner::hicon_to_rgba;
