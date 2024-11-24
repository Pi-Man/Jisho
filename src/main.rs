//#![windows_subsystem = "windows"]

use std::mem::size_of;

use resources::{IDI_ICON_JISHO, IDS_TOOLTIP};
use widestring::U16CString;

use window::popupwind::PopupWindow;
use window::Window;
use windows_sys::core::GUID;
use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Controls::{InitCommonControls, LoadIconMetric, LIM_SMALL};
use windows_sys::Win32::UI::Shell::{Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, NOTIFYICONDATAW_0, NOTIFYICON_VERSION_4};

mod resources;

mod window;

use window::mainwind::MainWindow;

const UWM_NOTIFYICONMESSAGE: u32 = WM_USER;

#[derive(Default)]
struct AppState {
    hinstance: HINSTANCE,
    m_hwnd: Option<MainWindow>,
    p_hwnd: Option<PopupWindow>,
}

#[derive(Default)]
struct Sense {
    defs: Vec<U16CString>,
    tags: Vec<U16CString>,
    see_also: Vec<U16CString>,
    info: Vec<U16CString>,
}

#[derive(Default)]
struct Entry {
    kanji: U16CString,
    reading: U16CString,
    senses: Vec<Sense>,
}

const LOOKUP_MODIFIERS: u16 = MOD_CONTROL as u16;
const LOOKUP_KEY: u16 = VK_D;

const SEARCH_MODIFIERS: u16 = MOD_CONTROL as u16;
const SEARCH_KEY: u16 = VK_D;

fn init_application(app: &mut AppState) { unsafe {
    app.hinstance =  GetModuleHandleW(std::ptr::null());
    InitCommonControls();
    MainWindow::register(app);
    PopupWindow::register(app);
}}

fn main() {

    let mut app = AppState::default();

    init_application(&mut app);

    app.m_hwnd = Some(unsafe { MainWindow::new(&mut app) });

    let mut notif_data = NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: app.m_hwnd.as_ref().map_or(0, |window| window.handle),
        uID: 0,
        uFlags: NIF_ICON | NIF_MESSAGE | NIF_SHOWTIP | NIF_TIP,
        uCallbackMessage: UWM_NOTIFYICONMESSAGE,
        hIcon: 0,
        szTip: [0; 128],
        dwState: 0,
        dwStateMask: 0,
        szInfo: [0; 256],
        Anonymous: NOTIFYICONDATAW_0 {uVersion: NOTIFYICON_VERSION_4},
        szInfoTitle: [0; 64],
        dwInfoFlags: 0,
        guidItem: GUID{data1: 0, data2: 0, data3: 0, data4: [0; 8]},
        hBalloonIcon: 0,
    };

    unsafe { LoadStringW(app.hinstance, IDS_TOOLTIP, notif_data.szTip.as_mut_ptr(), 128) };

    unsafe { LoadIconMetric(app.hinstance, IDI_ICON_JISHO as *const u16, LIM_SMALL, &mut notif_data.hIcon) };

    unsafe { Shell_NotifyIconW(NIM_ADD, &notif_data) };

    unsafe { RegisterHotKey(app.m_hwnd.as_ref().map_or(0, |window| window.handle), 0, LOOKUP_MODIFIERS as u32 | MOD_NOREPEAT, LOOKUP_KEY as u32) };

    unsafe { let mut msg = MSG {
        hwnd: 0,
        lParam: 0,
        message: 0,
        pt: POINT {x: 0, y: 0},
        time: 0,
        wParam: 0,
    };

    while GetMessageW(&mut msg, 0, 0, 0) != 0 {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }}

    unsafe { Shell_NotifyIconW(NIM_DELETE, &notif_data) };

}
