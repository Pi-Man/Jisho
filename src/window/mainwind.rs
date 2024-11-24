use std::{ffi::c_void, mem::size_of, ptr::{addr_of_mut, null}, thread, time::Duration};

use reqwest::blocking::Response;
use widestring::{u16cstr, U16CStr, U16CString};
use windows_sys::Win32::{Foundation::{HGLOBAL, HWND, LPARAM, LRESULT, POINT, WPARAM}, System::{DataExchange::{CloseClipboard, GetClipboardData, OpenClipboard}, Memory::{GlobalLock, GlobalUnlock}, Ole::CF_UNICODETEXT}, UI::{Input::KeyboardAndMouse::{MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, MAPVK_VK_TO_VSC, VK_C, VK_CONTROL}, Shell::NIN_SELECT, WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, GetCursorPos, GetSubMenu, GetWindowLongPtrW, LoadMenuW, PostMessageW, PostQuitMessage, RegisterClassW, SetForegroundWindow, SetWindowLongPtrW, TrackPopupMenu, CREATESTRUCTA, CW_USEDEFAULT, GWLP_USERDATA, TPM_BOTTOMALIGN, TPM_LEFTALIGN, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_HOTKEY, WM_QUIT, WM_RBUTTONUP, WNDCLASSW}}};

use crate::{resources::{IDM_MENU, IDM_NOTIF_QUIT}, window::Window, AppState, Entry, Sense, LOOKUP_KEY, LOOKUP_MODIFIERS, SEARCH_KEY, SEARCH_MODIFIERS, UWM_NOTIFYICONMESSAGE};

use super::popupwind::PopupWindow;

#[derive(Default)]
pub struct MainWindow {
    pub handle: HWND,
}

fn parse(json_text: String) -> Vec<Entry> {
    match json::parse(json_text.as_str()) {
        Err(e) => {
            println!("{:?}", e.to_string());
            Vec::<Entry>::new()
        }
        Ok(json) => {
            json["data"].members().map(|j| {
                Entry {
                    kanji: U16CString::from_str_truncate(j["japanese"][0]["word"].as_str().unwrap_or_default()),
                    reading: U16CString::from_str_truncate(j["japanese"][0]["reading"].as_str().unwrap_or_default()),
                    senses: j["senses"].members().map(|sense| 
                        Sense {
                            defs: sense["english_definitions"].members().map(|val| 
                                U16CString::from_str_truncate(val.as_str().unwrap_or_default())
                            ).collect(),
                            tags: sense["tags"].members().map(|val| 
                                U16CString::from_str_truncate(val.as_str().unwrap_or_default())
                            ).collect(),
                            see_also: sense["see_also"].members().map(|val| 
                                U16CString::from_str_truncate(val.as_str().unwrap_or_default())
                            ).collect(),
                            info: sense["info"].members().map(|val| 
                                U16CString::from_str_truncate(val.as_str().unwrap_or_default())
                            ).collect()
                        }
                    ).collect()
                }
            }).collect()
        },
    }
}

unsafe fn get_selection() -> String {
    macro_rules! key {
        ($key_code:expr, $flags:expr) => {
            INPUT{r#type: INPUT_KEYBOARD, Anonymous: INPUT_0 { ki: KEYBDINPUT {
                dwExtraInfo: 0,
                dwFlags: $flags,
                time: 0,
                wVk: $key_code,
                wScan: MapVirtualKeyW($key_code as u32, MAPVK_VK_TO_VSC) as u16
            }}}
        };
    }

    let a = [
        key!(VK_CONTROL, 0),
        key!(VK_C, 0),
        key!(VK_CONTROL, KEYEVENTF_KEYUP),
        key!(VK_C, KEYEVENTF_KEYUP),
    ];

    SendInput(a.len() as u32, a.as_ptr(), size_of::<INPUT>() as i32);

    thread::sleep(Duration::from_millis(100));

    if OpenClipboard(0) != 0 {
        let hclip = GetClipboardData(CF_UNICODETEXT as u32) as HGLOBAL;
        let contents_ptr = GlobalLock(hclip) as *mut u16;
        if !contents_ptr.is_null() {
            let contents = U16CStr::from_ptr_str(contents_ptr);
            let contents = contents.display().to_string();
            GlobalUnlock(hclip);
            CloseClipboard();
            contents
        }
        else {
            String::default()
        }
    }
    else {
        String::default()
    }

}

unsafe extern "system" fn windproc (hwnd: HWND, umsg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match umsg {
        WM_HOTKEY => {
            let modifiers = (lparam & 0xffff) as u16;
            let vkcode = ((lparam >> 16) & 0xffff) as u16;
            if (modifiers & LOOKUP_MODIFIERS) == LOOKUP_MODIFIERS && vkcode == LOOKUP_KEY {
                    
                let selection = get_selection();

                let url = if selection.is_ascii() {
                    format!("https://jisho.org/api/v1/search/words?keyword=\"{0}\"", selection.to_ascii_lowercase())
                }
                else {
                    format!("https://jisho.org/api/v1/search/words?keyword={0}", selection)
                };

                let response = reqwest::blocking::get(url).and_then(Response::text);

                match response {
                    Err(e) => {
                        println!("{:?}", e.to_string());
                    },
                    Ok(text) => {
                        let app = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState);
                        let entries = parse(text);
                        app.p_hwnd = Some(PopupWindow::new(app, entries));
                    },
                }
            }
            else if (modifiers & SEARCH_MODIFIERS) == SEARCH_MODIFIERS && vkcode == SEARCH_KEY {
                
            }
            0
        },
        WM_CREATE => {
            let createstruct = lparam as *mut CREATESTRUCTA;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*createstruct).lpCreateParams as isize);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        WM_COMMAND => {
            match wparam as u32 {
                IDM_NOTIF_QUIT => {
                    PostMessageW(hwnd, WM_QUIT, 0, 0);
                }
                _ => ()
            }
            0
        }
        UWM_NOTIFYICONMESSAGE => {
            let app = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState);
            match (lparam & 0xFFFF) as u32 {
                NIN_SELECT => {
                    println!("Notif Select");
                }
                WM_RBUTTONUP => {
                    let hmenu = LoadMenuW(app.hinstance, IDM_MENU as *const u16);
                    let hmenu = GetSubMenu(hmenu, 0);
                    let mut cursor_pos = POINT{x: 0, y: 0};
                    GetCursorPos(&mut cursor_pos);
                    SetForegroundWindow(hwnd);
                    TrackPopupMenu(hmenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, cursor_pos.x, cursor_pos.y, 0, hwnd, null());
                }
                _id => {
                    //println!("Notif {}", id);
                }
            }
            0
        }
        _ => DefWindowProcW(hwnd, umsg, wparam, lparam),
    }
}

impl MainWindow {
    pub unsafe fn new(app: &mut AppState) -> MainWindow {
        MainWindow {
            handle: CreateWindowExW(
                        0,
                        u16cstr!("JishoMain").as_ptr(),
                        u16cstr!("Jisho").as_ptr(),
                        0,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        0,
                        0,
                        0,
                        0,
                        app.hinstance,
                        addr_of_mut!(*app) as *mut c_void
                    )
        }
    }
}

impl Window for MainWindow {
    fn register(app: &AppState) {
        let main_class = WNDCLASSW {
            cbClsExtra: 0,
            cbWndExtra: 0,
            hCursor: 0,
            hIcon: 0,
            hInstance: app.hinstance,
            hbrBackground: 0,
            lpszClassName: u16cstr!("JishoMain").as_ptr(),
            lpszMenuName: null(),
            style: 0,
            lpfnWndProc: Some(windproc),
        };
        unsafe { RegisterClassW(&main_class) };
    }
    
    fn get_handle(&self) -> HWND {
        self.handle
    }
}