use winapi::{
    um::{
        winuser,
        wingdi,
        winbase::{ INFINITE },
        synchapi::{ CreateEventW, WaitForSingleObject },
        handleapi::{ CloseHandle },
    },
    shared::{
        windef,
        minwindef,
        winerror::{ SUCCEEDED },
    },
    ctypes::c_void,
};

use std::ptr;
use std::mem;

pub fn register_wndclass(class_name: &[u16]) -> bool {

    let mut winc = unsafe { mem::zeroed::<winuser::WNDCLASSW>() };
    winc.style = winuser::CS_HREDRAW | winuser::CS_VREDRAW;
    winc.lpfnWndProc = Some(window_procedure);
    winc.hIcon = unsafe { winuser::LoadIconW(ptr::null_mut(), winuser::IDI_APPLICATION) };
    winc.hCursor = unsafe { winuser::LoadCursorW(ptr::null_mut(), winuser::IDC_ARROW) };
    winc.hbrBackground = unsafe { wingdi::GetStockObject(wingdi::WHITE_BRUSH as i32) as windef::HBRUSH };
    winc.lpszClassName = class_name.as_ptr();

    unsafe {
        winuser::RegisterClassW(&winc) > 0
    }

}

pub fn create_window(class_name: &[u16], width: i32, height: i32) -> windef::HWND {

    unsafe {
        winuser::CreateWindowExW(
        0,
        class_name.as_ptr(),
        class_name.as_ptr(),
        winuser::WS_OVERLAPPEDWINDOW,
        0, 0, width, height,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        )
    }

}

pub extern "system" fn window_procedure(hwnd: windef::HWND, msg: minwindef::UINT, w_param: minwindef::WPARAM, l_param: minwindef::LPARAM) -> minwindef::LRESULT {

    match msg {
        winuser::WM_DESTROY => unsafe { winuser::PostQuitMessage(0) },
        _ => return unsafe { winuser::DefWindowProcW(hwnd, msg, w_param, l_param) },
    };

    0
}

pub fn show_window(hwnd: windef::HWND) {
    unsafe {
        winuser::ShowWindow(hwnd, winuser::SW_NORMAL);
        winuser::UpdateWindow(hwnd);
    }
}

pub fn quit_window(msg: *mut winuser::MSG) -> bool {
   let result = unsafe { winuser::GetMessageW(msg, ptr::null_mut(), 0, 0) } == 0;

   result
}
