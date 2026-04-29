#![windows_subsystem = "windows"]

use std::mem::zeroed;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

const CHAR_W: i32 = 14;
const CHAR_H: i32 = 22;
const FONT_HEIGHT: i32 = 20;
const FADE_ALPHA: u8 = 6;
const FRAME_INTERVAL_MS: u32 = 16;
const TIMER_ID: usize = 1;

const COLOR_BODY: u32 = 0x0040FF00;
const COLOR_HEAD: u32 = 0x00EBFFD9;

const POOL: &str =
    "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン0123456789";

struct Renderer {
    width: i32,
    height: i32,
    cols: usize,
    drops: Vec<f32>,
    speeds: Vec<f32>,
    last_drawn: Vec<i32>,
    pool: Vec<u16>,

    back_dc: HDC,
    back_bmp: HBITMAP,
    back_old: HGDIOBJ,

    fade_dc: HDC,
    fade_bmp: HBITMAP,
    fade_old: HGDIOBJ,

    font: HFONT,
    font_old: HGDIOBJ,

    rng: u64,
    initial_mouse: Option<(i32, i32)>,
}

impl Renderer {
    unsafe fn new(window_dc: HDC, w: i32, h: i32) -> Self {
        let back_dc = CreateCompatibleDC(window_dc);
        let back_bmp = CreateCompatibleBitmap(window_dc, w, h);
        let back_old = SelectObject(back_dc, back_bmp as HGDIOBJ);

        let black = GetStockObject(BLACK_BRUSH);
        let full = RECT { left: 0, top: 0, right: w, bottom: h };
        FillRect(back_dc, &full, black as _);

        let fade_dc = CreateCompatibleDC(window_dc);
        let fade_bmp = CreateCompatibleBitmap(window_dc, 1, 1);
        let fade_old = SelectObject(fade_dc, fade_bmp as HGDIOBJ);
        let one = RECT { left: 0, top: 0, right: 1, bottom: 1 };
        FillRect(fade_dc, &one, black as _);

        let font = create_font();
        let font_old = SelectObject(back_dc, font as HGDIOBJ);
        SetBkMode(back_dc, TRANSPARENT as i32);

        let cols = (w / CHAR_W).max(1) as usize;
        let rows = (h / CHAR_H).max(1);

        let mut rng: u64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEAD_BEEF_CAFE_F00D)
            | 1;

        let drops: Vec<f32> = (0..cols)
            .map(|_| rand_f32(&mut rng) * rows as f32)
            .collect();
        let speeds: Vec<f32> = (0..cols)
            .map(|_| 0.15 + rand_f32(&mut rng).powf(1.7) * 0.6)
            .collect();
        let last_drawn = vec![-1i32; cols];
        let pool: Vec<u16> = POOL.encode_utf16().collect();

        Renderer {
            width: w,
            height: h,
            cols,
            drops,
            speeds,
            last_drawn,
            pool,
            back_dc,
            back_bmp,
            back_old,
            fade_dc,
            fade_bmp,
            fade_old,
            font,
            font_old,
            rng,
            initial_mouse: None,
        }
    }

    unsafe fn step(&mut self) {
        let bf = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: FADE_ALPHA,
            AlphaFormat: 0,
        };
        AlphaBlend(
            self.back_dc, 0, 0, self.width, self.height,
            self.fade_dc, 0, 0, 1, 1,
            bf,
        );

        for i in 0..self.cols {
            self.drops[i] += self.speeds[i];
            let idx = self.drops[i] as i32;
            if idx != self.last_drawn[i] {
                self.last_drawn[i] = idx;
                let glyph = self.pool[(rand_u32(&mut self.rng) as usize) % self.pool.len()];
                let is_head = rand_u32(&mut self.rng) % 12 == 0;
                let color = if is_head { COLOR_HEAD } else { COLOR_BODY };
                SetTextColor(self.back_dc, color);
                let x = (i as i32) * CHAR_W;
                let y = idx * CHAR_H;
                if y + CHAR_H >= 0 && y < self.height {
                    let buf = [glyph];
                    TextOutW(self.back_dc, x, y, buf.as_ptr(), 1);
                }
            }
            if self.drops[i] * (CHAR_H as f32) > self.height as f32
                && rand_f32(&mut self.rng) > 0.975
            {
                self.drops[i] = 0.0;
                self.last_drawn[i] = -1;
            }
        }
    }

    unsafe fn present(&self, dst: HDC) {
        BitBlt(
            dst, 0, 0, self.width, self.height,
            self.back_dc, 0, 0,
            SRCCOPY,
        );
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.back_dc, self.font_old);
            SelectObject(self.back_dc, self.back_old);
            SelectObject(self.fade_dc, self.fade_old);
            DeleteObject(self.font as _);
            DeleteObject(self.back_bmp as _);
            DeleteObject(self.fade_bmp as _);
            DeleteDC(self.back_dc);
            DeleteDC(self.fade_dc);
        }
    }
}

fn rand_u32(s: &mut u64) -> u32 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    (*s >> 32) as u32
}

fn rand_f32(s: &mut u64) -> f32 {
    rand_u32(s) as f32 / (u32::MAX as f32)
}

unsafe fn create_font() -> HFONT {
    let mut face = [0u16; 32];
    for (i, c) in "Consolas".encode_utf16().enumerate() {
        face[i] = c;
    }
    let lf = LOGFONTW {
        lfHeight: -FONT_HEIGHT,
        lfWidth: 0,
        lfEscapement: 0,
        lfOrientation: 0,
        lfWeight: FW_BOLD as i32,
        lfItalic: 0,
        lfUnderline: 0,
        lfStrikeOut: 0,
        lfCharSet: DEFAULT_CHARSET as u8,
        lfOutPrecision: OUT_DEFAULT_PRECIS as u8,
        lfClipPrecision: CLIP_DEFAULT_PRECIS as u8,
        lfQuality: NONANTIALIASED_QUALITY as u8,
        lfPitchAndFamily: (FIXED_PITCH | FF_MODERN) as u8,
        lfFaceName: face,
    };
    CreateFontIndirectW(&lf)
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            let rptr = (*cs).lpCreateParams as isize;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, rptr);
            SetTimer(hwnd, TIMER_ID, FRAME_INTERVAL_MS, None);
            0
        }
        WM_TIMER => {
            let rptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
            if !rptr.is_null() {
                let r = &mut *rptr;
                r.step();
                let dc = GetDC(hwnd);
                r.present(dc);
                ReleaseDC(hwnd, dc);
            }
            0
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = zeroed();
            let dc = BeginPaint(hwnd, &mut ps);
            let rptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
            if !rptr.is_null() {
                let r = &*rptr;
                r.present(dc);
            }
            EndPaint(hwnd, &ps);
            0
        }
        WM_SETCURSOR => {
            SetCursor(null_mut());
            1
        }
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN => {
            PostMessageW(hwnd, WM_CLOSE, 0, 0);
            0
        }
        WM_MOUSEMOVE => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            let rptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
            if !rptr.is_null() {
                let r = &mut *rptr;
                match r.initial_mouse {
                    None => r.initial_mouse = Some((x, y)),
                    Some((ix, iy)) => {
                        if (x - ix).abs() > 5 || (y - iy).abs() > 5 {
                            PostMessageW(hwnd, WM_CLOSE, 0, 0);
                        }
                    }
                }
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, TIMER_ID);
            let rptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Renderer;
            if !rptr.is_null() {
                drop(Box::from_raw(rptr));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn run_saver() {
    unsafe {
        let hinstance = GetModuleHandleW(null());

        let class_name: Vec<u16> = "CodeRainSaver\0".encode_utf16().collect();
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance as _,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: GetStockObject(BLACK_BRUSH) as _,
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let h = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let probe_dc = GetDC(null_mut());
        let renderer = Box::new(Renderer::new(probe_dc, w, h));
        ReleaseDC(null_mut(), probe_dc);
        let renderer_ptr = Box::into_raw(renderer);

        let title: Vec<u16> = "Code Rain\0".encode_utf16().collect();
        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_POPUP | WS_VISIBLE,
            x, y, w, h,
            null_mut(),
            null_mut(),
            hinstance as _,
            renderer_ptr as _,
        );

        if hwnd.is_null() {
            drop(Box::from_raw(renderer_ptr));
            return;
        }

        ShowWindow(hwnd, SW_SHOW);
        SetForegroundWindow(hwnd);

        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args
        .get(1)
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "/s".to_string());

    let flag = mode.split(':').next().unwrap_or("/s");

    match flag {
        "/c" => {
            unsafe {
                let title: Vec<u16> = "Code Rain\0".encode_utf16().collect();
                let body: Vec<u16> =
                    "Code Rain has no settings.\0".encode_utf16().collect();
                MessageBoxW(
                    null_mut(),
                    body.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONINFORMATION,
                );
            }
        }
        "/p" => {
            // Mini preview pane in screensaver settings dialog — intentionally
            // a no-op. Windows will fall back to a static preview.
        }
        _ => run_saver(),
    }
}
