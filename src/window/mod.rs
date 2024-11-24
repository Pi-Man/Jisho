use std::ptr::null;

use windows_sys::Win32::{Foundation::{HINSTANCE, HWND, LPARAM, WPARAM}, UI::{Controls::{LoadIconMetric, LIM_SMALL, WC_BUTTONW, WC_EDITW}, WindowsAndMessaging::{CreateWindowExW, GetWindowLongPtrW, SendMessageW, BM_SETIMAGE, BS_ICON, GWLP_USERDATA, HICON, IMAGE_ICON, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_VISIBLE}}};

use crate::AppState;

pub mod mainwind;
pub mod popupwind;

pub trait Window {
    fn register(app: &AppState);
    fn get_handle(&self) -> HWND;
    fn get_app(&self) -> &mut AppState {
        unsafe { &mut *(GetWindowLongPtrW(self.get_handle(), GWLP_USERDATA) as *mut AppState) }
    }
    unsafe fn get_app_of(hwnd: HWND) -> *mut AppState {
        GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState
    }
    fn add_text(&self, id: isize, hinstance: HINSTANCE, text: *const u16, x: i32, y: i32, w: i32, h: i32, es_flags: u32) -> HWND {
        unsafe { CreateWindowExW(
            0,
            WC_EDITW,
            text,
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | es_flags,
            x,
            y,
            w,
            h,
            self.get_handle(),
            id,
            hinstance,
            null()
        ) }
    }

    fn add_icon_button(&self, id: isize, hinstance: HINSTANCE, icon: u32, x: i32, y: i32, size: i32) -> HWND { unsafe {
        let hwnd = CreateWindowExW(
            0,
            WC_BUTTONW,
            null(),
            WS_CHILD | WS_VISIBLE | BS_ICON as u32,
            x,
            y,
            size,
            size,
            self.get_handle(),
            id,
            hinstance,
            null()
        );
        let mut hicon: HICON = 0;
        LoadIconMetric(hinstance, icon as *const u16, LIM_SMALL, &mut hicon);
        SendMessageW(hwnd, BM_SETIMAGE, IMAGE_ICON as WPARAM, hicon as LPARAM);
        hwnd
    }}
}