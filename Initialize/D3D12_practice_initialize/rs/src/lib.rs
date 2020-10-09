use winapi::{
    um::{
        winuser::{ RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
                  LoadIconW, IDI_APPLICATION, LoadCursorW, IDC_ARROW,
                  CreateWindowExW, ShowWindow, SW_NORMAL, UpdateWindow,
                  GetMessageW, TranslateMessage, DispatchMessageW, MSG,
                  WM_DESTROY, PostQuitMessage, DefWindowProcW, WS_OVERLAPPEDWINDOW },
        wingdi::{ GetStockObject, WHITE_BRUSH },
        d3d12::*,
        d3d12sdklayers::*,
        d3dcommon::*,
        d3dcompiler::*,
        unknwnbase::{ IUnknown },
        winbase::{ INFINITE },
        synchapi::{ CreateEventW, WaitForSingleObject },
        handleapi::{ CloseHandle },
    },
    shared::{
        windef::{ HWND, HBRUSH },
        minwindef,
        winerror,
        ntdef::{ LUID, WCHAR },
        guiddef::*,
        dxgi,
        dxgi1_2::*,
        dxgi1_3,
        dxgi1_4::*,
        dxgi1_5::*,
        dxgi1_6::*,
        dxgiformat::*,
        dxgitype::*,
    },
    ctypes::c_void,
    Interface,

};

use std::ptr;
use std::mem;
use std::str;
use std::path::{ Path, PathBuf };
use std::ffi::CString;
use std::env;

pub fn create_dxgi_factory1<T: Interface>() -> Result<*mut T, winerror::HRESULT> {
    let mut obj = ptr::null_mut::<T>();
    let result = unsafe { dxgi::CreateDXGIFactory1(&T::uuidof(), get_pointer_of_self_object(&mut obj)) };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_dxgi_factory2<T: Interface>(Flags: minwindef::UINT) -> Result<*mut T, winerror::HRESULT> {
    let mut obj = ptr::null_mut::<T>();
    let result = unsafe { dxgi1_3::CreateDXGIFactory2(Flags, &T::uuidof(), get_pointer_of_self_object(&mut obj)) };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

fn get_pointer_of_self_object<T>(object: &mut T) -> *mut *mut winapi::ctypes::c_void {
    let mut_ref: &mut T = object;

    // next we need to convert the reference to a pointer
    let raw_ptr: *mut T = mut_ref as *mut T;

    // and the pointer type we can cast to the c_void type required by CreateWindowEx
    let void_ptr: *mut *mut winapi::ctypes::c_void = raw_ptr as *mut *mut winapi::ctypes::c_void;

    void_ptr
}
