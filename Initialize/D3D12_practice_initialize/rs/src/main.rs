use winapi::{
    um::{
        winuser::{ RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
                  LoadIconW, IDI_APPLICATION, LoadCursorW, IDC_ARROW,
                  CreateWindowExW, ShowWindow, SW_NORMAL, UpdateWindow,
                  GetMessageW, TranslateMessage, DispatchMessageW, MSG,
                  WM_DESTROY, PostQuitMessage, DefWindowProcW, WS_OVERLAPPEDWINDOW },
        wingdi::{ GetStockObject, WHITE_BRUSH },
        d3d12,
        d3d12sdklayers,
        d3dcommon,
    },
    shared::{
        windef::{ HWND, HBRUSH },
        minwindef::{ UINT, WPARAM, LPARAM, LRESULT },
        winerror::{ S_OK },
        guiddef::*,
    },
    ctypes::c_void,
};
use std::ptr;
use std::mem;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;

fn main() {
    unsafe {
        let class_name = encode("DX12Sample");
        if !register_wndclass(&class_name) {
            return;
        }

        let hwnd = create_window(&class_name);
        if hwnd.is_null() {
            return;
        }

        let mut dev: d3d12::ID3D12Device;


        // initialize Direct3D device
        let levels: [d3dcommon::D3D_FEATURE_LEVEL; 4] = [
            d3dcommon::D3D_FEATURE_LEVEL_12_1,
            d3dcommon::D3D_FEATURE_LEVEL_12_0,
            d3dcommon::D3D_FEATURE_LEVEL_11_1,
            d3dcommon::D3D_FEATURE_LEVEL_11_0
        ];

        let mut feature_level : d3dcommon::D3D_FEATURE_LEVEL = 0;

        const REFGUID: GUID = GUID {
            Data1: 0x189819f1,
            Data2: 0x1db6,
            Data3: 0x4b57,
            Data4: [ 0xbe, 0x54, 0x18, 0x21, 0x33, 0x9b, 0x85, 0xf7 ],
        };

        let mut pp_device = ptr::null_mut::<d3d12::ID3D12Device>();

        for lv in levels.iter() {

            if d3d12::D3D12CreateDevice(
                ptr::null_mut(),
                *lv,
                &REFGUID,
                &mut pp_device as *mut *mut d3d12::ID3D12Device as *mut *mut c_void
                )
                 == S_OK {
                println!("{}", "foooooo");

                feature_level = *lv;
			    break;
            }
        }

        println!("{}", feature_level);



        ShowWindow(hwnd, SW_NORMAL);
        UpdateWindow(hwnd);


        let mut msg = mem::MaybeUninit::uninit().assume_init();
        loop {
            if GetMessageW(&mut msg, ptr::null_mut(), 0, 0) == 0 {
                return;
            }
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);
        }
    }
}

fn encode(source: &str) -> Vec<u16> {
    source.encode_utf16().chain(Some(0)).collect()
}

unsafe fn register_wndclass(class_name: &[u16]) -> bool {
    let mut winc = mem::zeroed::<WNDCLASSW>();
    winc.style = CS_HREDRAW | CS_VREDRAW;
    winc.lpfnWndProc = Some(window_procedure);
    winc.hIcon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);
    winc.hCursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);
    winc.hbrBackground = GetStockObject(WHITE_BRUSH as i32) as HBRUSH;
    winc.lpszClassName = class_name.as_ptr();

    RegisterClassW(&winc) > 0
}

unsafe fn create_window(class_name: &[u16]) -> HWND {
    CreateWindowExW(
        0,
        class_name.as_ptr(),
        encode("DX12Sample").as_ptr(),
        WS_OVERLAPPEDWINDOW,
        0, 0, WINDOW_WIDTH, WINDOW_HEIGHT,
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
        ptr::null_mut(),
    )
}

unsafe extern "system" fn window_procedure(hwnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => PostQuitMessage(0),
        _ => return DefWindowProcW(hwnd, msg, w_param, l_param),
    };
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn some_test() {
        let levels: [d3dcommon::D3D_FEATURE_LEVEL; 4] = [
            d3dcommon::D3D_FEATURE_LEVEL_12_1,
            d3dcommon::D3D_FEATURE_LEVEL_12_0,
            d3dcommon::D3D_FEATURE_LEVEL_11_1,
            d3dcommon::D3D_FEATURE_LEVEL_11_0
        ];

        println!("{:?}", levels);
    }
}
