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
    },
    shared::{
        windef::{ HWND, HBRUSH },
        minwindef::{ UINT, WPARAM, LPARAM, LRESULT },
        winerror::{ S_OK, DXGI_ERROR_NOT_FOUND },
        ntdef::{ LUID, WCHAR },
        guiddef::*,
        dxgi::*,
        dxgi1_2::*,
        dxgi1_3::*,
        dxgi1_4::*,
        dxgiformat::*,
        dxgitype::*,
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

        let mut device = ptr::null_mut::<ID3D12Device>();
        let mut dxgi_factory = ptr::null_mut::<IDXGIFactory4>();


        // initialize Direct3D device
        let levels: [D3D_FEATURE_LEVEL; 4] = [
            D3D_FEATURE_LEVEL_12_1,
            D3D_FEATURE_LEVEL_12_0,
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0
        ];

        let mut feature_level : D3D_FEATURE_LEVEL = 0;

        for lv in levels.iter() {

            if D3D12CreateDevice(
                ptr::null_mut(),
                *lv,
                &IID_ID3D12Device,
                &mut device as *mut *mut ID3D12Device as *mut *mut c_void
                )
                 == S_OK {
                feature_level = *lv;
			    break;
            }
        }

        // return value 0 is S_OK
        let result = CreateDXGIFactory(&IID_IDXGIFactory, &mut dxgi_factory as *mut *mut IDXGIFactory4 as *mut *mut c_void);

        // iterate adapter to use
        let mut tmp_adapter = ptr::null_mut::<IDXGIAdapter>();

        let mut i = 0;

        while IDXGIFactory::EnumAdapters(&*dxgi_factory, i, &mut tmp_adapter as *mut *mut IDXGIAdapter) != DXGI_ERROR_NOT_FOUND {
            i += 1;

            let mut p_desc = DXGI_ADAPTER_DESC {
                Description: [0; 128],
                VendorId: 0,
                DeviceId: 0,
                SubSysId: 0,
                Revision: 0,
                DedicatedVideoMemory: 0,
                DedicatedSystemMemory: 0,
                SharedSystemMemory: 0,
                AdapterLuid: LUID {
                    LowPart: 0,
                    HighPart: 0,
                },
            };

            IDXGIAdapter::GetDesc(&*tmp_adapter, &mut p_desc);

            if p_desc.Description.to_vec() != encode("NVIDIA") {
                // println!("{:?}", encode("NVIDIA"));
                // println!("{:?}", p_desc.Description.to_vec());
                tmp_adapter = tmp_adapter;

                break;
            }

        }

        println!("{:?}", &tmp_adapter);



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
        let levels: [D3D_FEATURE_LEVEL; 4] = [
            D3D_FEATURE_LEVEL_12_1,
            D3D_FEATURE_LEVEL_12_0,
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0
        ];

        println!("{:?}", levels);
    }
}
