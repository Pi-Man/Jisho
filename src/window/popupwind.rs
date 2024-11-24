



use std::{cmp::max, ffi::c_void, mem::{size_of, MaybeUninit}, ptr::{addr_of, addr_of_mut, null}};

use widestring::u16cstr;
use windows_sys::Win32::{Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, SIZE, WPARAM}, Graphics::Gdi::{CreateFontIndirectW, GetObjectW, GetStockObject, DC_BRUSH, LOGFONTW}, UI::{HiDpi::GetDpiForWindow, Input::{KeyboardAndMouse::EnableWindow, RegisterRawInputDevices, RAWINPUTDEVICE, RIDEV_INPUTSINK}, Shell::ShellExecuteW, WindowsAndMessaging::{CalculatePopupWindowPosition, CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetWindowRect, RegisterClassW, SendMessageW, SetForegroundWindow, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, BN_CLICKED, CREATESTRUCTA, CS_DROPSHADOW, ES_CENTER, ES_LEFT, ES_READONLY, ES_RIGHT, GWLP_USERDATA, LWA_ALPHA, SWP_NOZORDER, SW_SHOW, SW_SHOWNORMAL, TPM_LEFTALIGN, TPM_TOPALIGN, WM_ACTIVATEAPP, WM_COMMAND, WM_CREATE, WM_GETFONT, WM_INPUT, WM_SETFONT, WNDCLASSW, WS_CLIPCHILDREN, WS_EX_LAYERED, WS_POPUPWINDOW}}};

use crate::{resources::{IDI_ICON_JISHO, IDI_ICON_KIDNEY, IDI_ICON_LEFT, IDI_ICON_RIGHT}, window::Window, AppState, Entry};

const BTN_KIDNEY: isize = 2048;
const BTN_JISHO: isize = 2049;
const BTN_PREV: isize = 2050;
const BTN_NEXT: isize = 2051;

#[derive(Default)]
pub struct PopupWindow {
    pub handle: HWND,
    entries: Vec<Entry>,
    index: usize,

    prev_btn: HWND,
    next_btn: HWND,

    kanji_ec: HWND,
    reading_ec: HWND,
    defs_ec: Vec<HWND>,
}

impl PopupWindow {
    unsafe fn display_entry(&mut self, app: &AppState) {

        if self.entries.is_empty() {return}

        if self.kanji_ec != 0 {
            DestroyWindow(self.kanji_ec);
            self.kanji_ec = 0;
        }
        if self.reading_ec != 0 {
            DestroyWindow(self.reading_ec);
            self.reading_ec = 0;
        }
        self.defs_ec.iter().for_each(|ec| {
            DestroyWindow(*ec);
        });
        self.defs_ec.clear();
    
        unsafe fn set_font_size(hwnd: HWND, font_size: i32) {
            let hfont = SendMessageW(hwnd, WM_GETFONT, 0, 0);
            let mut logfont = MaybeUninit::<LOGFONTW>::zeroed();
            GetObjectW(hfont, size_of::<LOGFONTW>() as i32, logfont.as_mut_ptr() as *mut c_void);
            let mut logfont = logfont.assume_init();
            logfont.lfHeight = font_size;
            let hfont = CreateFontIndirectW(&logfont);
            SendMessageW(hwnd, WM_SETFONT, hfont as usize, 0);
        }

        const WIDTH: f64 = 250.0;
        const HEADER_FONT_SIZE: f64 = 28.0;
        const DEF_FONT_SIZE: f64 = 12.0;
        const MARGIN_SZIE: f64 = 5.0;
        const BUTTON_SIZE: f64 = 24.0;
    
        let dpi = GetDpiForWindow(self.handle);
        let pppt = dpi as f64 / 72.0;
        let window_width: i32 = (WIDTH * pppt).round() as i32;
        let margin: i32 = (MARGIN_SZIE * pppt).round() as i32;
        let header_height: i32 = (HEADER_FONT_SIZE * pppt).round() as i32;
        let break_height: i32 = (MARGIN_SZIE * pppt).round() as i32;
        let def_height: i32 = (DEF_FONT_SIZE * pppt).round() as i32;
        let button_height: i32 = (BUTTON_SIZE * pppt).round() as i32;

        let mut y = margin;

        EnableWindow(self.prev_btn, if self.index > 0 {1} else {0});
        EnableWindow(self.next_btn, if self.index < self.entries.len() - 1 {1} else {0});

        y += button_height + break_height;
        
        self.kanji_ec = self.add_text(1024, app.hinstance, self.entries[self.index].kanji.as_ptr(), margin, y, window_width - margin * 2, header_height, ES_CENTER as u32 | ES_READONLY as u32);
        set_font_size(self.kanji_ec, header_height);

        y += header_height;
        
        self.reading_ec = self.add_text(1025, app.hinstance, self.entries[self.index].reading.as_ptr(), margin, y, window_width - margin * 2, header_height, ES_CENTER as u32 | ES_READONLY as u32);
        set_font_size(self.reading_ec, header_height);

        y += header_height + break_height;

        for i in 0..self.entries[self.index].senses.len() {
            for j in 0..self.entries[self.index].senses[i].defs.len() {
                let hwnd = self.add_text(1026 + i as isize, app.hinstance, self.entries[self.index].senses[i].defs[j].as_ptr(), margin, y, window_width - margin * 2, def_height, ES_LEFT as u32 | ES_READONLY as u32);
                self.defs_ec.push(hwnd);
                set_font_size(hwnd, def_height);
                y += def_height;
            }
            for j in 0..self.entries[self.index].senses[i].tags.len() {
                let hwnd = self.add_text(1026 + i as isize, app.hinstance, self.entries[self.index].senses[i].tags[j].as_ptr(), margin, y, window_width - margin * 2, def_height, ES_RIGHT as u32 | ES_READONLY as u32);
                self.defs_ec.push(hwnd);
                set_font_size(hwnd, def_height);
                y += def_height;
            }
            for j in 0..self.entries[self.index].senses[i].see_also.len() {
                let hwnd = self.add_text(1026 + i as isize, app.hinstance, self.entries[self.index].senses[i].see_also[j].as_ptr(), margin, y, window_width - margin * 2, def_height, ES_RIGHT as u32 | ES_READONLY as u32);
                self.defs_ec.push(hwnd);
                set_font_size(hwnd, def_height);
                y += def_height;
            }
            for j in 0..self.entries[self.index].senses[i].info.len() {
                let hwnd = self.add_text(1026 + i as isize, app.hinstance, self.entries[self.index].senses[i].info[j].as_ptr(), margin, y, window_width - margin * 2, def_height, ES_RIGHT as u32 | ES_READONLY as u32);
                self.defs_ec.push(hwnd);
                set_font_size(hwnd, def_height);
                y += def_height;
            }
            y += margin;
        }

        let mut rect = RECT {
            left: 0,
            right: 0,
            bottom: 0,
            top: 0,
        };
        GetWindowRect(self.handle, &mut rect);

        let window_height: i32 = y;
        let size = SIZE {
            cx: window_width,
            cy: window_height,
        };
    
        let mut window_pos = MaybeUninit::<RECT>::uninit();
        CalculatePopupWindowPosition(addr_of!(rect) as *const POINT, &size, TPM_LEFTALIGN | TPM_TOPALIGN, null(), window_pos.as_mut_ptr());
        let window_pos = window_pos.assume_init();
    
        SetWindowPos(self.handle, 0, window_pos.left, window_pos.top, window_width, window_height, SWP_NOZORDER);
    }

    pub unsafe fn display_next(&mut self, app: *mut AppState) {
        self.index += 1;
        if self.index >= self.entries.len() {
            self.index = self.entries.len();
        }
        self.display_entry(&*app);
    }

    pub unsafe fn display_prev(&mut self, app: *mut AppState) {
        self.index -= 1;
        if self.index <= 0 {
            self.index = 0;
        }
        self.display_entry(&*app);
    }

    pub unsafe fn new(app: &mut AppState, entries: Vec<Entry>) -> PopupWindow {

        let mut point = POINT{x: 0, y: 0};
        GetCursorPos(&mut point);
    
        let mut window = PopupWindow::default();
        window.handle = CreateWindowExW(
                WS_EX_LAYERED,
                u16cstr!("JishoPopup").as_ptr(),
                u16cstr!("Jisho").as_ptr(),
                WS_POPUPWINDOW | WS_CLIPCHILDREN,
                point.x,
                point.y,
                0,
                0,
                app.m_hwnd.as_ref().map_or(0, |window| window.handle),
                0,
                app.hinstance,
                addr_of_mut!(*app) as *mut c_void
        );
        window.entries = entries;
    
        let mouse = RAWINPUTDEVICE {
            usUsagePage: 1,
            usUsage: 2,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: window.handle,
        };

        const WIDTH: f64 = 250.0;
        const MARGIN_SZIE: f64 = 5.0;
        const BUTTON_SIZE: f64 = 24.0;
    
        let dpi = GetDpiForWindow(window.handle);
        let pppt = dpi as f64 / 72.0;
        let window_width: i32 = (WIDTH * pppt).round() as i32;
        let margin: i32 = (MARGIN_SZIE * pppt).round() as i32;
        let button_height: i32 = (BUTTON_SIZE * pppt).round() as i32;

        let y = margin;
        let mut x = margin;

        window.add_icon_button(BTN_KIDNEY, app.hinstance, IDI_ICON_KIDNEY, x, y, button_height);
        x += button_height + margin;
        window.add_icon_button(BTN_JISHO, app.hinstance, IDI_ICON_JISHO, x, y, button_height);
        window.prev_btn = window.add_icon_button(BTN_PREV, app.hinstance, IDI_ICON_LEFT, window_width / 2 - margin / 2 - button_height, y, button_height);
        window.next_btn = window.add_icon_button(BTN_NEXT, app.hinstance, IDI_ICON_RIGHT, window_width / 2 + margin / 2, y, button_height);

        window.display_entry(app);

        RegisterRawInputDevices(&mouse, 1, size_of::<RAWINPUTDEVICE>() as u32);
    
        SetLayeredWindowAttributes(window.handle, 0, 0xFF, LWA_ALPHA);
    
        ShowWindow(window.handle, SW_SHOW);
        SetForegroundWindow(window.handle);

        window
    }

    unsafe extern "system" fn windproc(hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match umsg {
            WM_CREATE => {
                let createstruct = lparam as *mut CREATESTRUCTA;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*createstruct).lpCreateParams as isize);
                0
            }
            WM_INPUT => {
    
                let mut point = POINT {
                    x: 0,
                    y: 0,
                };
    
                GetCursorPos(&mut point);
                let point = point;
    
                let mut rect = MaybeUninit::<RECT>::uninit();
                GetWindowRect(hwnd, rect.as_mut_ptr());
                let rect = rect.assume_init();
    
                let dx = if point.x < rect.left {rect.left - point.x} else if point.x > rect.right {point.x - rect.right} else {0};
                let dy = if point.y < rect.top {rect.top - point.y} else if point.y > rect.bottom {point.y - rect.bottom} else {0};
    
                let dist = max(dx, dy);
    
                if dist > 0xFF {
                    DestroyWindow(hwnd);
                }
                else {
                    SetLayeredWindowAttributes(hwnd, 0, 0xFF - dist as u8, LWA_ALPHA);
                }
                0
            }
            WM_COMMAND => {
                match (wparam >> 16) as u32 {
                    BN_CLICKED => {
                        match (wparam & 0xFFFF) as isize {
                            BTN_KIDNEY => {
                                ShellExecuteW(0, u16cstr!("Open").as_ptr(), u16cstr!("https://discord.gg/jlss").as_ptr(), null(), null(), SW_SHOWNORMAL);
                            }
                            BTN_JISHO => {
                                ShellExecuteW(0, u16cstr!("Open").as_ptr(), u16cstr!("https://jisho.org").as_ptr(), null(), null(), SW_SHOWNORMAL);
                            }
                            BTN_PREV => {
                                let app = PopupWindow::get_app_of(hwnd);
                                (&mut*app).p_hwnd.as_mut().unwrap().display_prev(app);
                            }
                            BTN_NEXT => {
                                let app = PopupWindow::get_app_of(hwnd);
                                (&mut*app).p_hwnd.as_mut().unwrap().display_next(app);
                            }
                            _ => ()
                        }
                        
                    }
                    _ => ()
                }
                0
            }
            WM_ACTIVATEAPP => {
                if wparam == 0 {
                    DestroyWindow(hwnd);
                }
                0
            }
            _ => DefWindowProcW(hwnd, umsg, wparam, lparam)
        }
    }
}

impl Window for PopupWindow {
    fn register(app: &AppState) {
        
        let popup_class = WNDCLASSW {
            cbClsExtra: 0,
            cbWndExtra: 0,
            hCursor: 0,
            hIcon: 0,
            hInstance: app.hinstance,
            hbrBackground: unsafe { GetStockObject(DC_BRUSH) },
            lpszClassName: u16cstr!("JishoPopup").as_ptr(),
            lpszMenuName: null(),
            style: CS_DROPSHADOW,
            lpfnWndProc: Some(PopupWindow::windproc),
        };
    
        unsafe { RegisterClassW(&popup_class) };
    }
    
    fn get_handle(&self) -> HWND {
        self.handle
    }
}