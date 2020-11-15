use winapi::{
    um::{
        winuser,
        wingdi,
        d3d12,
        d3d12sdklayers,
        d3dcommon,
        d3dcompiler,
        unknwnbase,
        winbase,
        synchapi,
        handleapi,
    },
    shared::{
        windef,
        minwindef,
        winerror,
        ntdef,
        guiddef::*,
        dxgi,
        dxgi1_2,
        dxgi1_3,
        dxgi1_4::*,
        dxgi1_5::*,
        dxgi1_6,
        dxgiformat,
        dxgitype::*,
    },
    ctypes,
    Interface,

};

use std::ptr;
use std::mem;
use std::str;
use std::path;
use std::ffi::CString;
use std::env;
use image::{ GenericImageView, DynamicImage };

#[derive(Debug, Clone, Copy)]
pub struct XMFLOAT3 {
    pub x: f32,
    pub y: f32,
    pub z: f32
}
#[derive(Debug, Clone, Copy)]
pub struct XMFLOAT2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    pub position: XMFLOAT3,
    pub uv: XMFLOAT2,
}
#[derive(Debug, Clone)]
pub struct Image {
    pub width: u64,
    pub height: u32,
    pub format: dxgiformat::DXGI_FORMAT,
    pub row_pitch: usize,
    pub slice_pitch: usize,
    pub alignmented_row_pitch: u32,
    pub alignmented_slice_pitch: u64,
    pub raw_pointer: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TexMetadata {
    pub width: u64,
    pub height: u64,
    pub depth: u64,
    pub array_size: u64,
    pub mip_levels: u64,
    pub misc_flags: u64,
    pub misc_flags2: u64,
    pub format: u64,
    pub dimension: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct CommittedResource {
    pub pHeapProperties: *const d3d12::D3D12_HEAP_PROPERTIES,
    pub HeapFlags: d3d12::D3D12_HEAP_FLAGS,
    pub pResourceDesc: *const d3d12::D3D12_RESOURCE_DESC,
    pub InitialResourceState: d3d12::D3D12_RESOURCE_STATES,
    pub pOptimizedClearValue: *const d3d12::D3D12_CLEAR_VALUE
}

// #[derive(Debug, Clone, Copy, Default)]
pub struct BufferResources<T> {
    pub buffer_view: T,
    pub buffer_object: *const d3d12::ID3D12Resource,
}


pub fn create_dxgi_factory1<T: Interface>() -> Result<*mut T, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<T>();
    let result = unsafe {
        dxgi::CreateDXGIFactory1(&T::uuidof(), get_pointer_of_interface(&mut obj))
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

pub fn create_dxgi_factory2<T: Interface>(Flags: minwindef::UINT) -> Result<*mut T, winerror::HRESULT> {
    let mut obj = ptr::null_mut::<T>();
    let result = unsafe {
        dxgi1_3::CreateDXGIFactory2(Flags, &T::uuidof(), get_pointer_of_interface(&mut obj))
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
                    get_pointer_of_interface(&mut obj)
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

        if p_desc.Description.to_vec() != utf16_to_vec("NVIDIA") {

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
        CreateCommandAllocator(type_, &d3d12::ID3D12CommandAllocator::uuidof(), get_pointer_of_interface(&mut obj))
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
            get_pointer_of_interface(&mut obj)
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
            get_pointer_of_interface(&mut obj)
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
                                    ppSwapChain: *mut *mut dxgi1_2::IDXGISwapChain1) {

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
}

pub fn create_descriptor_heap(device: *mut d3d12::ID3D12Device, pDescriptorHeapDesc: *const d3d12::D3D12_DESCRIPTOR_HEAP_DESC) -> Result<*mut d3d12::ID3D12DescriptorHeap, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12DescriptorHeap>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateDescriptorHeap(
            pDescriptorHeapDesc,
            &d3d12::ID3D12DescriptorHeap::uuidof(),
            get_pointer_of_interface(&mut obj)
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
            swapchain.as_ref().unwrap().GetBuffer(i as u32, &d3d12::ID3D12Resource::uuidof(), get_pointer_of_interface(&mut back_buffers[i as usize]));
        }

        unsafe {
            device.as_ref().unwrap().CreateRenderTargetView(back_buffers[i as usize], pDesc, handle)
        }

        handle.ptr += unsafe {
            device.as_ref().unwrap().GetDescriptorHandleIncrementSize(d3d12::D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize
        }
    }

    back_buffers
}

pub fn create_fence(device: *mut d3d12::ID3D12Device, InitialValue: i32, Flags: d3d12::D3D12_FENCE_FLAGS) -> Result<*mut d3d12::ID3D12Fence, winerror::HRESULT> {

    let mut obj = ptr::null_mut::<d3d12::ID3D12Fence>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateFence(
            InitialValue as u64,
            Flags,
            &d3d12::ID3D12Fence::uuidof(),
            get_pointer_of_interface(&mut obj)
        )
    };

    match result {
        winerror::S_OK => Ok(obj),
        _ => Err(result)
    }
}

fn create_buffer_map<T>(device: *mut d3d12::ID3D12Device, comitted_resource: CommittedResource, resource: Vec<T>) -> *mut d3d12::ID3D12Resource {

    let mut buffer = std::ptr::null_mut::<d3d12::ID3D12Resource>();

    let mut result = unsafe {
                device.as_ref().unwrap().
                CreateCommittedResource(
                    comitted_resource.pHeapProperties,
                    comitted_resource.HeapFlags,
                    comitted_resource.pResourceDesc,
                    comitted_resource.InitialResourceState,
                    comitted_resource.pOptimizedClearValue,
                    &d3d12::ID3D12Resource::uuidof(),
                    get_pointer_of_interface(&mut buffer)
            )
    };

    // buffer map
    let mut buffer_map = std::ptr::null_mut::<Vec<T>>();

    // map buffer to GPU
    result = unsafe {
        buffer.as_ref().unwrap().
        Map(0, std::ptr::null_mut(), get_pointer_of_interface(&mut buffer_map))
    };
    unsafe {
        buffer_map.copy_from_nonoverlapping(resource.as_ptr().cast::<Vec<T>>(), std::mem::size_of_val(&resource) )
    };
    unsafe {
        buffer.as_ref().unwrap().
        Unmap(0, std::ptr::null_mut() )
    };

    buffer
}

pub fn create_vertex_buffer_resources(device: *mut d3d12::ID3D12Device, comitted_resource: CommittedResource, resource: Vec<Vertex>) -> BufferResources<d3d12::D3D12_VERTEX_BUFFER_VIEW> {

    let tmp_resource = resource.clone();

    let buffer = create_buffer_map(device, comitted_resource, resource);

    let buffer_view = d3d12::D3D12_VERTEX_BUFFER_VIEW {
        BufferLocation : unsafe { buffer.as_ref().unwrap().GetGPUVirtualAddress() },
        SizeInBytes : (tmp_resource.len() * mem::size_of::<Vertex>()) as u32,
        StrideInBytes : std::mem::size_of_val(&tmp_resource[0]) as u32,
    };

    BufferResources {
        buffer_view: buffer_view,
        buffer_object: buffer
    }
}

pub fn create_index_buffer_resources(device: *mut d3d12::ID3D12Device, comitted_resource: CommittedResource, resource: Vec<u16>) -> BufferResources<d3d12::D3D12_INDEX_BUFFER_VIEW> {

    let tmp_resource = resource.clone();

    // reuse vertex buffer desc
    let pResourceDesc = comitted_resource.pResourceDesc as *mut d3d12::D3D12_RESOURCE_DESC;
    unsafe {
        (*pResourceDesc).Width = (std::mem::size_of_val(&resource) * &resource.len()) as u64
    };

    let buffer = create_buffer_map(device, comitted_resource, resource);

    let buffer_view = d3d12::D3D12_INDEX_BUFFER_VIEW {
        BufferLocation : unsafe { buffer.as_ref().unwrap().GetGPUVirtualAddress() },
        Format : dxgiformat::DXGI_FORMAT_R16_UINT,
        SizeInBytes : (tmp_resource.len() * mem::size_of::<u16>()) as u32,
    };

    BufferResources {
        buffer_view: buffer_view,
        buffer_object: buffer
    }
}

pub fn create_texture_buffer_from_file(path: &str) -> Image {

    let img = image::open(get_relative_file_path(path)).unwrap();

    let color_type = img.color();

    let bits_per_pixel = match color_type {

        image::ColorType::Rgb8 => mem::size_of::<image::Rgba<u8>>(),

        image::ColorType::Rgba8 => mem::size_of::<image::Rgba<u8>>(),

        _ => mem::size_of::<image::Rgb<u8>>()
    };

    let row_pitch = bits_per_pixel * (img.width() as usize);

    let slice_pitch = row_pitch * (img.height() as usize);

    let format = match color_type {

        image::ColorType::Rgb8 => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,

        image::ColorType::Rgba8 => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,

        _ => dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM
    };


    let alignmented_row_pitch = if img.width() % d3d12::D3D12_TEXTURE_DATA_PITCH_ALIGNMENT == 0 {
        row_pitch as u32
    } else {
        (row_pitch as u32 + d3d12::D3D12_TEXTURE_DATA_PITCH_ALIGNMENT) - (row_pitch as u32 % d3d12::D3D12_TEXTURE_DATA_PITCH_ALIGNMENT)
    };

    let alignmented_slice_pitch = alignmented_row_pitch * img.height();

    Image {
        width: img.width() as u64,
        height: img.height() as u32,
        format: format,
        row_pitch: row_pitch,
        slice_pitch: slice_pitch,
        alignmented_row_pitch: alignmented_row_pitch,
        alignmented_slice_pitch: alignmented_slice_pitch as u64,
        raw_pointer: img.to_bytes(),
    }
}


pub fn create_shader_resource(path: &str, pEntrypoint: &str, pTarget: &str, error_blob: *mut d3dcommon::ID3DBlob) -> Result<*mut d3dcommon::ID3DBlob, winerror::HRESULT> {

    let mut shader_blob = std::ptr::null_mut::<d3dcommon::ID3DBlob>();

    let result = unsafe {
        d3dcompiler::D3DCompileFromFile(
            path_to_wide_str(path).as_ptr() as *const u16,
            std::ptr::null_mut(),
            d3dcompiler::D3D_COMPILE_STANDARD_FILE_INCLUDE,
            CString::new(pEntrypoint).unwrap().as_ptr(),
            CString::new(pTarget).unwrap().as_ptr(),
            d3dcompiler::D3DCOMPILE_DEBUG | d3dcompiler::D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut shader_blob,
            error_blob as *mut *mut d3dcommon::ID3D10Blob
        )
    };

    // notify compilation status
    const FILE_NOT_FOUND: i32 = winerror::ERROR_FILE_NOT_FOUND as i32;
    const PATH_NOT_FOUND: i32 = winerror::ERROR_PATH_NOT_FOUND as i32;
    match result {
        FILE_NOT_FOUND => Err(result),
        PATH_NOT_FOUND => Err(result),
        winerror::S_OK =>  Ok(shader_blob),
        _ => {
            // output compilation error message
            let error_str = unsafe {
                std::string::String::from_raw_parts(
                error_blob.as_ref().unwrap().GetBufferPointer().cast::<u8>(),
                error_blob.as_ref().unwrap().GetBufferSize(),
                error_blob.as_ref().unwrap().GetBufferSize())
            };

            println!("{:?}", error_str);

            Err(-1)

        }
    }
}

pub fn create_root_signature(device: *mut d3d12::ID3D12Device, error_blob: *mut d3dcommon::ID3DBlob) -> *mut d3d12::ID3D12RootSignature {

    let mut root_signature = std::ptr::null_mut::<d3d12::ID3D12RootSignature>();

    let mut root_signature_desc: d3d12::D3D12_ROOT_SIGNATURE_DESC = unsafe { mem::zeroed() };
    root_signature_desc.Flags = d3d12::D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;

    // texture settings
    let descriptor_range = d3d12::D3D12_DESCRIPTOR_RANGE {
        NumDescriptors: 1,
        RangeType: d3d12::D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        BaseShaderRegister: 0,
        RegisterSpace: 0,
        OffsetInDescriptorsFromTableStart: d3d12::D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
    };

    let mut root_param = d3d12::D3D12_ROOT_PARAMETER {
        ParameterType: d3d12::D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
        ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_PIXEL,
        u: unsafe { mem::zeroed() },
    };
    * unsafe { root_param.u.DescriptorTable_mut() } = d3d12::D3D12_ROOT_DESCRIPTOR_TABLE {
        NumDescriptorRanges: 1,
        pDescriptorRanges: &descriptor_range,
    };

    let sampler_desc = d3d12::D3D12_STATIC_SAMPLER_DESC {
        AddressU: d3d12::D3D12_TEXTURE_ADDRESS_MODE_WRAP,
        AddressV: d3d12::D3D12_TEXTURE_ADDRESS_MODE_WRAP,
        AddressW: d3d12::D3D12_TEXTURE_ADDRESS_MODE_WRAP,
        BorderColor: d3d12::D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK,
        Filter: d3d12::D3D12_FILTER_MIN_MAG_MIP_POINT,
        MaxLOD: d3d12::D3D12_FLOAT32_MAX,
        MinLOD: 0.0,
        ComparisonFunc: d3d12::D3D12_COMPARISON_FUNC_NEVER,
        ShaderVisibility: d3d12::D3D12_SHADER_VISIBILITY_PIXEL,
        MipLODBias: 0.0,
        MaxAnisotropy: 0,
        ShaderRegister: 0,
        RegisterSpace: 0
    };

    root_signature_desc.pParameters = &root_param;
    root_signature_desc.NumParameters = 1;

    root_signature_desc.pStaticSamplers = &sampler_desc;
    root_signature_desc.NumStaticSamplers = 1;
    // texture settings


    // create root signature binary
    let mut root_signature_blob = std::ptr::null_mut::<d3dcommon::ID3DBlob>();

    let mut result = unsafe {
            d3d12::D3D12SerializeRootSignature(
                &root_signature_desc,
                d3d12::D3D_ROOT_SIGNATURE_VERSION_1_0,
                &mut root_signature_blob,
                error_blob as *mut *mut d3dcommon::ID3D10Blob
            )
    };

    result = unsafe {
        device.as_ref().unwrap().
        CreateRootSignature(
            0,
            root_signature_blob.as_ref().unwrap().GetBufferPointer(),
            root_signature_blob.as_ref().unwrap().GetBufferSize(),
            &d3d12::ID3D12RootSignature::uuidof(),
            get_pointer_of_interface(&mut root_signature)
        )
    };

    unsafe {
        root_signature_blob.as_ref().unwrap().Release()
    };

    root_signature
}

pub fn create_pipeline_state(device: *mut d3d12::ID3D12Device, gr_pipeline: d3d12::D3D12_GRAPHICS_PIPELINE_STATE_DESC) -> *mut d3d12::ID3D12PipelineState {

    let mut pipeline_state = std::ptr::null_mut::<d3d12::ID3D12PipelineState>();

    let result = unsafe {
        device.as_ref().unwrap().
        CreateGraphicsPipelineState(
            &gr_pipeline,
            &d3d12::ID3D12PipelineState::uuidof(),
            get_pointer_of_interface(&mut pipeline_state)
        )
    };

    pipeline_state
}

pub fn set_viewport(width: i32, height: i32) -> d3d12::D3D12_VIEWPORT {
    let mut viewport: d3d12::D3D12_VIEWPORT = unsafe { mem::zeroed() };
    viewport.Width = width as f32;
    viewport.Height = height as f32;
    viewport.TopLeftX = 0.0;
    viewport.TopLeftY = 0.0;
    viewport.MaxDepth = 1.0;
    viewport.MinDepth = 0.0;

    viewport
}

pub fn set_scissor_rect(width: i32, height: i32) -> d3d12::D3D12_RECT {
    let mut scissor_rect: d3d12::D3D12_RECT = unsafe { mem::zeroed() };
    scissor_rect.top = 0;
    scissor_rect.left = 0;
    scissor_rect.right = scissor_rect.left + width;
    scissor_rect.bottom = scissor_rect.top + height;

    scissor_rect
}

pub fn enable_debug_layer(is_debug: bool) {

    if !is_debug {
        return;
    }

    let mut debug_controller = ptr::null_mut::<d3d12sdklayers::ID3D12Debug>();

    if winerror::SUCCEEDED(
        unsafe {
                d3d12::D3D12GetDebugInterface(
                    &d3d12sdklayers::ID3D12Debug::uuidof(),
                    get_pointer_of_interface(&mut debug_controller)
                )
            }
        )
    {
        unsafe {
            debug_controller.as_ref().unwrap().EnableDebugLayer();
        }
    }
}

pub fn report_live_objects(device: *mut d3d12::ID3D12Device, is_debug: bool) {

    if !is_debug {
        return;
    }

    let mut debug_interface = ptr::null_mut::<d3d12sdklayers::ID3D12DebugDevice>();

    if winerror::SUCCEEDED(
        unsafe {
                device.as_ref().unwrap().
                QueryInterface(
                    &d3d12sdklayers::ID3D12DebugDevice::uuidof(),
                    get_pointer_of_interface(&mut debug_interface)
                )
            }
        ) {
        unsafe {
            debug_interface.as_ref().unwrap().ReportLiveDeviceObjects(d3d12sdklayers::D3D12_RLDO_DETAIL | d3d12sdklayers::D3D12_RLDO_IGNORE_INTERNAL);
            debug_interface.as_ref().unwrap().Release();
        };
    }
}

pub fn utf16_to_vec(source: &str) -> Vec<u16> {
    source.encode_utf16().chain(Some(0)).collect()
}

fn get_relative_file_path(s: &str) -> path::PathBuf {
    let relative_path = path::Path::new(s);
    let pwd = env::current_dir().unwrap();
    let absolute_path = pwd.join(relative_path);

    absolute_path
}

fn path_to_wide_str(s: &str) -> Vec<u16> {
    let wide_str = utf16_to_vec(get_relative_file_path(s).to_str().unwrap());

    wide_str
}

pub fn get_pointer_of_interface<T>(object: &mut T) -> *mut *mut ctypes::c_void {
    // we need to convert the reference to a pointer
    let raw_ptr = object as *mut T;

    // and the pointer type we can cast to the c_void type required T
    let void_ptr = raw_ptr as *mut *mut ctypes::c_void;

    // in one liner
    // void_ptr as *mut *mut T as *mut *mut ctypes::c_void

    void_ptr
}


// #[cfg(test)]
// mod tests {
//     use super::*;


//     #[test]
//     fn some_test() {

//         let slice = vec![
//             Vertex {
//                 position: XMFLOAT3 { x: -0.4, y: -0.7, z: 0.0 },
//                 uv: XMFLOAT2 { x: 0.0, y: 1.0 },
//             },
//             Vertex {
//                 position: XMFLOAT3 { x: -0.4, y: -0.7, z: 0.0 },
//                 uv: XMFLOAT2 { x: 0.0, y: 1.0 },
//             },
//         ];


//         println!("{:?}", slice.len() * mem::size_of::<Vertex>());

//     }
// }
