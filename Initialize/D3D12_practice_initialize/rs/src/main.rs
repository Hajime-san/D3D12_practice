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
        unknwnbase::{ IUnknown },
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
        dxgi1_5::*,
        dxgi1_6::*,
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
        let mut dxgi_factory = ptr::null_mut::<IDXGIFactory6>();
        let mut swapchain = ptr::null_mut::<IDXGISwapChain1>();


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

        // return value 0(HRESULT->S_OK) is ok
        let mut result = CreateDXGIFactory(&IID_IDXGIFactory, &mut dxgi_factory as *mut *mut IDXGIFactory6 as *mut *mut c_void);

        // iterate adapter to use
        let mut tmp_adapter = ptr::null_mut();

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

        // create command list, allocator
        let mut cmd_allocator = ptr::null_mut();
        let mut cmd_list = ptr::null_mut();

        result = device.as_ref().unwrap().CreateCommandAllocator(
            D3D12_COMMAND_LIST_TYPE_DIRECT,
            &IID_ID3D12CommandAllocator,
            &mut cmd_allocator as *mut *mut ID3D12CommandAllocator as *mut *mut c_void
        );

        result = device.as_ref().unwrap().CreateCommandList(
            0,
            D3D12_COMMAND_LIST_TYPE_DIRECT,
            cmd_allocator,
            ptr::null_mut(),
            &IID_ID3D12GraphicsCommandList,
            &mut cmd_list as *mut *mut ID3D12GraphicsCommandList as *mut *mut c_void
        );

        // create commnad queue

        let mut cmd_queue = ptr::null_mut::<ID3D12CommandQueue>();

        let mut cmd_queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Flags : D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask : 0,
            Priority : D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32,
            Type : D3D12_COMMAND_LIST_TYPE_DIRECT,
        };

        result = device.as_ref().unwrap().CreateCommandQueue(
            &mut cmd_queue_desc,
            &IID_ID3D12CommandQueue,
            &mut cmd_queue as *mut *mut ID3D12CommandQueue as *mut *mut c_void
        );

        // create swapchain
        let swapchain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width : WINDOW_WIDTH as u32,
            Height : WINDOW_HEIGHT as u32,
            Format : DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo : 0,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count : 1,
                Quality : 0,
            },
            BufferUsage : DXGI_USAGE_BACK_BUFFER,
            BufferCount : 2,
            Scaling : DXGI_SCALING_STRETCH,
            SwapEffect : DXGI_SWAP_EFFECT_FLIP_DISCARD,
            AlphaMode : DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags : DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
        };

        // function CreateSwapChainForHwnd 1st argument pDevice is command queue object in IDXGIFactory6
        result = dxgi_factory.as_ref().unwrap().CreateSwapChainForHwnd(
            cmd_queue as *mut IUnknown,
            hwnd,
            &swapchain_desc,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut swapchain
        );

        println!("{:?}", result);



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
