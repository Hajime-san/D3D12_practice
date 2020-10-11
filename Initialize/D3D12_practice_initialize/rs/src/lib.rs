use winapi::{
    um::{
        winuser::{ RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
                  LoadIconW, IDI_APPLICATION, LoadCursorW, IDC_ARROW,
                  CreateWindowExW, ShowWindow, SW_NORMAL, UpdateWindow,
                  GetMessageW, TranslateMessage, DispatchMessageW, MSG,
                  WM_DESTROY, PostQuitMessage, DefWindowProcW, WS_OVERLAPPEDWINDOW },
        wingdi::{ GetStockObject, WHITE_BRUSH },
        d3d12,
        d3d12sdklayers::*,
        d3dcommon,
        d3dcompiler::*,
        unknwnbase,
        winbase::{ INFINITE },
        synchapi::{ CreateEventW, WaitForSingleObject },
        handleapi::{ CloseHandle },
    },
    shared::{
        windef,
        minwindef,
        winerror,
        ntdef::{ LUID, WCHAR },
        guiddef::*,
        dxgi,
        dxgi1_2,
        dxgi1_3,
        dxgi1_4::*,
        dxgi1_5::*,
        dxgi1_6,
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
    let result = unsafe {
        dxgi::CreateDXGIFactory1(&T::uuidof(), get_pointer_of_self_object(&mut obj))
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_dxgi_factory2<T: Interface>(Flags: minwindef::UINT) -> Result<*mut T, winerror::HRESULT> {
    let mut obj = ptr::null_mut::<T>();
    let result = unsafe {
        dxgi1_3::CreateDXGIFactory2(Flags, &T::uuidof(), get_pointer_of_self_object(&mut obj))
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_d3d12_device() -> Result<*mut d3d12::ID3D12Device, winerror::HRESULT> {

    let levels: [d3dcommon::D3D_FEATURE_LEVEL; 4] = [
        d3dcommon::D3D_FEATURE_LEVEL_12_1,
        d3dcommon::D3D_FEATURE_LEVEL_12_0,
        d3dcommon::D3D_FEATURE_LEVEL_11_1,
        d3dcommon::D3D_FEATURE_LEVEL_11_0
    ];

    let mut obj = ptr::null_mut::<d3d12::ID3D12Device>();

    for lv in levels.iter() {

        if unsafe {
                d3d12::D3D12CreateDevice(
                ptr::null_mut(),
                *lv, &d3d12::ID3D12Device::uuidof(),
                get_pointer_of_self_object(&mut obj)
                )
                == winerror::S_OK
            }
            {
                break;
        }
    }

    match obj.is_null() {
        true => Err(winerror::S_FALSE),
        _ => Ok(obj)
    }
}

// get adapter to use manually
pub fn get_adapter(dxgi_factory: *mut dxgi::IDXGIFactory) -> Result<*mut dxgi::IDXGIAdapter, winerror::HRESULT> {

    let mut adapter = ptr::null_mut::<dxgi::IDXGIAdapter>();

    let mut i = 0;

    while unsafe {
        dxgi_factory.as_ref().unwrap().
        EnumAdapters(i, &mut adapter as *mut *mut dxgi::IDXGIAdapter)
        } != winerror::DXGI_ERROR_NOT_FOUND {
        i += 1;

        let mut p_desc: dxgi::DXGI_ADAPTER_DESC = unsafe { mem::zeroed() };

        unsafe {
            adapter.as_ref().unwrap().
            GetDesc(&mut p_desc)
        };

        if p_desc.Description.to_vec() != utf16_to_vector("NVIDIA") {

            adapter = adapter;

            break;
        }

    }

    match adapter.is_null() {
        true => Err(winerror::S_FALSE),
        _ => Ok(adapter)
    }
}

pub fn create_command_allocator(device: *mut d3d12::ID3D12Device, type_: d3d12::D3D12_COMMAND_LIST_TYPE) -> Result<*mut d3d12::ID3D12CommandAllocator, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12CommandAllocator>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateCommandAllocator(type_, &d3d12::ID3D12CommandAllocator::uuidof(), get_pointer_of_self_object(&mut obj))
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_command_list(device: *mut d3d12::ID3D12Device, nodeMask: u32, type_: d3d12::D3D12_COMMAND_LIST_TYPE, pCommandAllocator: *mut d3d12::ID3D12CommandAllocator, pInitialState: *mut d3d12::ID3D12PipelineState) -> Result<*mut d3d12::ID3D12GraphicsCommandList, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12GraphicsCommandList>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateCommandList(
            nodeMask,
            type_,
            pCommandAllocator,
            pInitialState,
            &d3d12::ID3D12GraphicsCommandList::uuidof(),
            get_pointer_of_self_object(&mut obj)
        )
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_command_queue(device: *mut d3d12::ID3D12Device, pDesc: *const d3d12::D3D12_COMMAND_QUEUE_DESC) -> Result<*mut d3d12::ID3D12CommandQueue, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12CommandQueue>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateCommandQueue(
            pDesc,
            &d3d12::ID3D12CommandQueue::uuidof(),
            get_pointer_of_self_object(&mut obj)
        )
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_swap_chain_for_hwnd(dxgi_factory: *mut dxgi1_6::IDXGIFactory6,
                                    pDevice: *mut d3d12::ID3D12CommandQueue,
                                    hWnd: windef::HWND,
                                    pDesc: *const dxgi1_2::DXGI_SWAP_CHAIN_DESC1,
                                    pFullscreenDesc: *mut dxgi1_2::DXGI_SWAP_CHAIN_FULLSCREEN_DESC,
                                    pRestrictToOutput: *mut dxgi::IDXGIOutput,
                                    ppSwapChain: *mut *mut dxgi1_2::IDXGISwapChain1) -> Result<*mut d3d12::ID3D12CommandQueue, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12CommandQueue>();

    let result = unsafe {
        dxgi_factory.as_ref().unwrap().
        CreateSwapChainForHwnd(
            pDevice as *mut unknwnbase::IUnknown,
            hWnd,
            pDesc,
            pFullscreenDesc,
            pRestrictToOutput,
            ppSwapChain
        )
     };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_descriptor_heap(device: *mut d3d12::ID3D12Device, pDescriptorHeapDesc: *const d3d12::D3D12_DESCRIPTOR_HEAP_DESC) -> Result<*mut d3d12::ID3D12DescriptorHeap, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12DescriptorHeap>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateDescriptorHeap(
            pDescriptorHeapDesc,
            &d3d12::ID3D12DescriptorHeap::uuidof(),
            get_pointer_of_self_object(&mut obj)
        )
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_back_buffer(device: *mut d3d12::ID3D12Device, swapchain: *mut dxgi1_2::IDXGISwapChain1, swapchain_desc: dxgi1_2::DXGI_SWAP_CHAIN_DESC1, descriotor_heap: *mut d3d12::ID3D12DescriptorHeap, pDesc: *const d3d12::D3D12_RENDER_TARGET_VIEW_DESC) -> Vec<*mut d3d12::ID3D12Resource> {

    // bind render target view heap to swap chain buffer
    let mut back_buffers = vec![std::ptr::null_mut::<d3d12::ID3D12Resource>(); swapchain_desc.BufferCount as usize];

    let mut handle = unsafe { descriotor_heap.as_ref().unwrap().GetCPUDescriptorHandleForHeapStart() };

    for i in 0..swapchain_desc.BufferCount {
        unsafe {
            swapchain.as_ref().unwrap().GetBuffer(i as u32, &d3d12::ID3D12Resource::uuidof(), get_pointer_of_self_object(&mut back_buffers[i as usize]));
        }

        unsafe {
            device.as_ref().unwrap().CreateRenderTargetView(back_buffers[i as usize], std::ptr::null_mut(), handle)
        }

        handle.ptr += unsafe {
                        device.as_ref().unwrap().GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize
                      }
    }

    back_buffers
}

fn get_pointer_of_self_object<T>(object: &mut T) -> *mut *mut winapi::ctypes::c_void {
    // we need to convert the reference to a pointer
    let raw_ptr = object as *mut T;

    // and the pointer type we can cast to the c_void type required T
    let void_ptr = raw_ptr as *mut *mut winapi::ctypes::c_void;

    // in one liner
    // void_ptr as *mut *mut T as *mut *mut winapi::ctypes::c_void

    void_ptr
}

fn utf16_to_vector(source: &str) -> Vec<u16> {
    source.encode_utf16().chain(Some(0)).collect()
}
