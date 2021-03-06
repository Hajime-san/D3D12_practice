use winapi::{
    um::{
        winuser::{ GetMessageW, TranslateMessage, DispatchMessageW },
        d3d12::*,
        d3dcommon::*,
        winbase::{ INFINITE },
        synchapi::{ CreateEventW, WaitForSingleObject },
        handleapi::{ CloseHandle },
    },
    shared::{
        minwindef::{ FLOAT },
        dxgi::*,
        dxgi1_2::*,
        dxgi1_3::*,
        dxgi1_5::*,
        dxgi1_6::*,
        dxgiformat::*,
        dxgitype::*,
    },
    ctypes,
};

use std::ptr;
use std::mem;
use std::ffi::CString;

pub mod lib;
pub mod win;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const DEBUG: bool = true;
const D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING: u32 = 0x1688;

enum BOOL {
    FALSE = 0,
    TRUE = 1,
}

fn main() {
    let class_name = lib::utf16_to_vec("DX12Sample");
    if !win::register_wndclass(&class_name) {
        return;
    }

    let hwnd = win::create_window(&class_name, WINDOW_WIDTH, WINDOW_HEIGHT);
    if hwnd.is_null() {
        return;
    }

    // return value 0(HRESULT->S_OK) is ok
    let mut result = -1;


    let mut dxgi_factory = ptr::null_mut();
    let mut swapchain = ptr::null_mut(); // IDXGISwapChain4


    if DEBUG {
        dxgi_factory = lib::create_dxgi_factory2::<IDXGIFactory6>(DXGI_CREATE_FACTORY_DEBUG).unwrap();
    } else {
        //dxgi_factory = lib::create_dxgi_factory1::<IDXGIFactory1>().unwrap();
    }

    // enable debug layer
    lib::enable_debug_layer(DEBUG);

    // device
    let d3d12_device = lib::create_d3d12_device().unwrap();

    // create command list, allocator
    let cmd_allocator = lib::create_command_allocator(d3d12_device, D3D12_COMMAND_LIST_TYPE_DIRECT).unwrap();
    let cmd_list = lib::create_command_list(d3d12_device, 0, D3D12_COMMAND_LIST_TYPE_DIRECT, cmd_allocator, ptr::null_mut()).unwrap();

    // create commnad queue
    let cmd_queue_desc = D3D12_COMMAND_QUEUE_DESC {
        Flags : D3D12_COMMAND_QUEUE_FLAG_NONE,
        NodeMask : 0,
        Priority : D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as i32,
        Type : D3D12_COMMAND_LIST_TYPE_DIRECT,
    };

    let cmd_queue = lib::create_command_queue(d3d12_device, &cmd_queue_desc).unwrap();

    // create swapchain
    let swapchain_desc1 = DXGI_SWAP_CHAIN_DESC1 {
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

    lib::create_swap_chain_for_hwnd(
        dxgi_factory,
        cmd_queue,
        hwnd,
        &swapchain_desc1,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut swapchain
    );

    // create Render Target View //

    // create discriptor heap
    let heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
        Type : D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        NodeMask : 0,
        NumDescriptors : 2,
        Flags : D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
    };

    let rtv_heaps = lib::create_descriptor_heap(d3d12_device, &heap_desc).unwrap();

    // SRGB render target view
	let rtv_desc = D3D12_RENDER_TARGET_VIEW_DESC {
        Format: DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
        u: unsafe { mem::zeroed() },
    };

    // bind render target view heap to swap chain buffer
    let back_buffers = lib::create_back_buffer(d3d12_device, swapchain, swapchain_desc1, rtv_heaps, &rtv_desc);

    // create vertices
    let vertices  = vec![
        lib::Vertex {
            position: lib::XMFLOAT3 { x: -0.4, y: -0.7, z: 0.0 },
            uv: lib::XMFLOAT2 { x: 0.0, y: 1.0 },
        },
        lib::Vertex {
            position: lib::XMFLOAT3 { x: -0.4, y: 0.7, z: 0.0 },
            uv: lib::XMFLOAT2 { x: 0.0, y: 0.0 },
        },
        lib::Vertex {
            position: lib::XMFLOAT3 { x: 0.4, y: -0.7, z: 0.0 },
            uv: lib::XMFLOAT2 { x: 1.0, y: 1.0 },
        },
        lib::Vertex {
            position: lib::XMFLOAT3 { x: 0.4, y: 0.7, z: 0.0 },
            uv: lib::XMFLOAT2 { x: 1.0, y: 0.0 },
        }
    ];

    // create vertex buffer

    // settings of vertex heap
    let vertex_buffer_heap_prop = D3D12_HEAP_PROPERTIES {
        Type : D3D12_HEAP_TYPE_UPLOAD,
        CPUPageProperty : D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
        MemoryPoolPreference : D3D12_MEMORY_POOL_UNKNOWN,
        CreationNodeMask: 0,
        VisibleNodeMask: 0,
    };

    // vertex buffer object
    let vertex_buffer_resource_desc = D3D12_RESOURCE_DESC {
        Dimension : D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width : (std::mem::size_of_val(&vertices) * &vertices.len()) as u64,
        Height : 1,
        DepthOrArraySize : 1,
        MipLevels : 1,
        Format : DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count : 1,
            Quality: 0,
        },
        Flags : D3D12_RESOURCE_FLAG_NONE,
        Layout : D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
    };

    let comitted_resource = lib::CommittedResource {
        pHeapProperties: &vertex_buffer_heap_prop,
        HeapFlags: D3D12_HEAP_FLAG_NONE,
        pResourceDesc: &vertex_buffer_resource_desc,
        InitialResourceState: D3D12_RESOURCE_STATE_GENERIC_READ,
        pOptimizedClearValue: std::ptr::null_mut(),
    };

    // create indices
    let indices = vec![
        0, 1, 2,
        2, 1, 3
    ];

    // create vertex resources
    let vertex_buffer = lib::create_vertex_buffer_resources(d3d12_device, comitted_resource, vertices.clone());

    let index_buffer = lib::create_index_buffer_resources(d3d12_device, comitted_resource, indices.clone());

    // create shader object
    let shader_error_blob = std::ptr::null_mut::<ID3DBlob>();
    let vertex_shader_blob = lib::create_shader_resource("shaders\\VertexShader.hlsl", "BasicVS", "vs_5_0", shader_error_blob).unwrap();
    let pixel_shader_blob = lib::create_shader_resource("shaders\\PixelShader.hlsl", "BasicPS", "ps_5_0", shader_error_blob).unwrap();

    // vertex layout
    let input_element: [D3D12_INPUT_ELEMENT_DESC; 2] = [
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: CString::new("POSITION").unwrap().into_raw(),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: CString::new("TEXCOORD").unwrap().into_raw(),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        },
    ];

    // create root signature
    let root_signature = lib::create_root_signature(d3d12_device, shader_error_blob);

    // create graphics pipeline
    let mut gr_pipeline: D3D12_GRAPHICS_PIPELINE_STATE_DESC = unsafe { mem::zeroed() };

    // set shader
    gr_pipeline.pRootSignature = root_signature;
    gr_pipeline.VS.pShaderBytecode = unsafe { vertex_shader_blob.as_ref().unwrap().GetBufferPointer() };
    gr_pipeline.VS.BytecodeLength = unsafe { vertex_shader_blob.as_ref().unwrap().GetBufferSize() };
    gr_pipeline.PS.pShaderBytecode = unsafe { pixel_shader_blob.as_ref().unwrap().GetBufferPointer() };
    gr_pipeline.PS.BytecodeLength = unsafe { pixel_shader_blob.as_ref().unwrap().GetBufferSize() };

    // sample mask
    gr_pipeline.SampleMask = D3D12_DEFAULT_SAMPLE_MASK;

    // culling, filling
    gr_pipeline.RasterizerState.CullMode = D3D12_CULL_MODE_NONE;
    gr_pipeline.RasterizerState.FillMode = D3D12_FILL_MODE_SOLID;
    gr_pipeline.RasterizerState.DepthClipEnable = BOOL::TRUE as i32;
    gr_pipeline.RasterizerState.FrontCounterClockwise = BOOL::FALSE as i32;
	gr_pipeline.RasterizerState.DepthBias = D3D12_DEFAULT_DEPTH_BIAS as i32;
	gr_pipeline.RasterizerState.DepthBiasClamp = D3D12_DEFAULT_DEPTH_BIAS_CLAMP;
	gr_pipeline.RasterizerState.SlopeScaledDepthBias = D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS;
	gr_pipeline.RasterizerState.AntialiasedLineEnable = BOOL::FALSE as i32;
	gr_pipeline.RasterizerState.ForcedSampleCount = 0;
    gr_pipeline.RasterizerState.ConservativeRaster = D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF;

    gr_pipeline.DepthStencilState.DepthEnable = BOOL::FALSE as i32;
	gr_pipeline.DepthStencilState.StencilEnable = BOOL::FALSE as i32;

    // blend mode
    gr_pipeline.BlendState.AlphaToCoverageEnable = BOOL::FALSE as i32;
    gr_pipeline.BlendState.IndependentBlendEnable = BOOL::FALSE as i32;

    // render target blend settings
    let mut render_target_blend_desc: D3D12_RENDER_TARGET_BLEND_DESC = unsafe { mem::zeroed() };
    render_target_blend_desc.BlendEnable = BOOL::FALSE as i32;
    render_target_blend_desc.LogicOpEnable = BOOL::FALSE as i32;
    render_target_blend_desc.RenderTargetWriteMask = D3D12_COLOR_WRITE_ENABLE_ALL as u8;

    gr_pipeline.BlendState.RenderTarget[0] = render_target_blend_desc;

    // bind input layout
    gr_pipeline.InputLayout.pInputElementDescs = &input_element as *const _;
    gr_pipeline.InputLayout.NumElements = input_element.len() as u32;

    // way to express triangle
    gr_pipeline.IBStripCutValue = D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED;

    // primitive topology setting
    gr_pipeline.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;

    // render target settings
    gr_pipeline.NumRenderTargets = 1;
    gr_pipeline.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM_SRGB;

    // anti aliasing
    gr_pipeline.RasterizerState.MultisampleEnable = BOOL::FALSE as i32;
    gr_pipeline.SampleDesc.Count = 1;
    gr_pipeline.SampleDesc.Quality = 0;

    // create grahphics pipeline state object
    let pipeline_state = lib::create_pipeline_state(d3d12_device, gr_pipeline);

    // viewport setting
    let viewport = lib::set_viewport(WINDOW_WIDTH, WINDOW_HEIGHT);

    // scissor rectangle setting
    let scissor_rect = lib::set_scissor_rect(WINDOW_WIDTH, WINDOW_HEIGHT);

    // create intermediate texture buffer for uploade resource
    let mut texture = lib::get_texture_data_from_file("assets\\images\\ultimate.png");

    let texture_buffer_heap_prop = D3D12_HEAP_PROPERTIES {
        Type : D3D12_HEAP_TYPE_UPLOAD,
        CPUPageProperty : D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
        MemoryPoolPreference : D3D12_MEMORY_POOL_UNKNOWN,
        CreationNodeMask: 0,
        VisibleNodeMask: 0,
    };


    let mut texture_buffer_resource_desc = D3D12_RESOURCE_DESC {
        Dimension : D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width : texture.alignmented_slice_pitch,
        Height : 1,
        DepthOrArraySize : 1,
        MipLevels : 1,
        Format : DXGI_FORMAT_UNKNOWN,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count : 1,
            Quality: 0,
        },
        Flags : D3D12_RESOURCE_FLAG_NONE,
        Layout : D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
    };

    let mut intermediate_buffer = std::ptr::null_mut::<ID3D12Resource>();

    result = unsafe {
        d3d12_device.as_ref().unwrap().
        CreateCommittedResource(
                &texture_buffer_heap_prop,
                D3D12_HEAP_FLAG_NONE,
                &texture_buffer_resource_desc,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                std::ptr::null_mut(),
                &IID_ID3D12Resource,
                &mut intermediate_buffer as *mut *mut ID3D12Resource as *mut *mut ctypes::c_void
        )
    };

    // create buffer for copy source to destination
    let texture_buffer_destination_prop = D3D12_HEAP_PROPERTIES {
        Type : D3D12_HEAP_TYPE_DEFAULT,
        CPUPageProperty : D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
        MemoryPoolPreference : D3D12_MEMORY_POOL_UNKNOWN,
        CreationNodeMask: 0,
        VisibleNodeMask: 0,
    };

    texture_buffer_resource_desc.Format = texture.format;
    texture_buffer_resource_desc.Width = texture.width as u64;
    texture_buffer_resource_desc.Height = texture.height;
    texture_buffer_resource_desc.DepthOrArraySize = 1;
    texture_buffer_resource_desc.MipLevels = 1;
    texture_buffer_resource_desc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;
    texture_buffer_resource_desc.Layout = D3D12_TEXTURE_LAYOUT_UNKNOWN;

    let mut texture_buffer = std::ptr::null_mut::<ID3D12Resource>();

    result = unsafe {
        d3d12_device.as_ref().unwrap().
        CreateCommittedResource(
            &texture_buffer_destination_prop,
            D3D12_HEAP_FLAG_NONE,
            &texture_buffer_resource_desc,
            D3D12_RESOURCE_STATE_COPY_DEST,
            std::ptr::null_mut(),
            &IID_ID3D12Resource,
            &mut texture_buffer as *mut *mut ID3D12Resource as *mut *mut ctypes::c_void
        )
    };

    // buffer map
    let mut buffer_map = std::ptr::null_mut::<u8>();

    // map buffer to GPU
    result = unsafe {
        intermediate_buffer.as_ref().unwrap().
        Map(0, std::ptr::null_mut(), lib::get_pointer_of_interface(&mut buffer_map))
    };


    unsafe {
        let mut tmp_pointer = texture.raw_pointer.as_mut_ptr().cast::<u8>();

        for _ in 0..texture.height {

            buffer_map.copy_from_nonoverlapping(tmp_pointer, texture.alignmented_row_pitch as usize);

            tmp_pointer = tmp_pointer.offset(texture.row_pitch as isize);

            buffer_map = buffer_map.offset(texture.alignmented_row_pitch as isize);
        }
    };

    unsafe {
        intermediate_buffer.as_ref().unwrap().
        Unmap(0, std::ptr::null_mut() )
    };

    // copy source description
    let mut copy_src = D3D12_TEXTURE_COPY_LOCATION {
        pResource: intermediate_buffer,
        Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
        u: unsafe { mem::zeroed() },
    };
    * unsafe { copy_src.u.PlacedFootprint_mut() } = D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
            Offset: 0,
            Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                Format: texture.format,
                Width: texture.width as u32,
                Height: texture.height,
                Depth: 1,
                RowPitch: texture.alignmented_row_pitch as u32,
            }
    };

    // copy destination
    let mut copy_dest = D3D12_TEXTURE_COPY_LOCATION {
        pResource: texture_buffer,
        Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
        u: unsafe { mem::zeroed() },
    };
    * unsafe { copy_dest.u.SubresourceIndex_mut() } = 0;

    // handle fence
    let mut current_frame: u64 = 0;

    // create fence
    let fence = lib::create_fence(d3d12_device, current_frame as i32, D3D12_FENCE_FLAG_NONE).unwrap();

    {

        current_frame += 1;

        unsafe {
            cmd_list.as_ref().unwrap().CopyTextureRegion(&copy_dest, 0, 0, 0, &copy_src, std::ptr::null_mut())
        };


        let mut texture_barrier_desc = D3D12_RESOURCE_BARRIER {
            Type : D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags : D3D12_RESOURCE_BARRIER_FLAG_NONE,
            u: unsafe { mem::zeroed() },
        };
        * unsafe { texture_barrier_desc.u.Transition_mut() } = D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource : texture_buffer,
            Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: D3D12_RESOURCE_STATE_COPY_DEST,
            StateAfter: D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
        };

        unsafe {
            cmd_list.as_ref().unwrap().ResourceBarrier(1, &texture_barrier_desc);
        };
        unsafe {
            cmd_list.as_ref().unwrap().Close();
        };


        let cmd_list_arrays = [ cmd_list.cast::<ID3D12CommandList>() ];

        unsafe {
            cmd_queue.as_ref().unwrap().ExecuteCommandLists(1, &cmd_list_arrays[0]);
        };

        unsafe {
            cmd_queue.as_ref().unwrap().Signal(fence, current_frame);
        };

        if unsafe { fence.as_ref().unwrap().GetCompletedValue() } != current_frame {

            let event = unsafe { CreateEventW(ptr::null_mut(), 0, 0, ptr::null_mut()) };

            unsafe {
                fence.as_ref().unwrap().SetEventOnCompletion(current_frame, event);
            };

            unsafe {
                WaitForSingleObject(event, INFINITE);
            };

            unsafe {
                CloseHandle(event);
            };
        }

        unsafe { cmd_allocator.as_ref().unwrap().Reset(); };

        unsafe { cmd_list.as_ref().unwrap().Reset(cmd_allocator, ptr::null_mut()); };

    };

    // cbv, srv, uav desctriptor heap
    let texture_view_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
        Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
        NodeMask: 0,
        NumDescriptors: 1,
        Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
    };

    let mut texture_desc_heap = lib::create_descriptor_heap(d3d12_device, &texture_view_heap_desc).unwrap();

	// shader resource view
    let mut shader_resource_view_desc = D3D12_SHADER_RESOURCE_VIEW_DESC {
        Format: texture.format,
        Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
        ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D,
        u: unsafe { mem::zeroed() },
    };
    unsafe { shader_resource_view_desc.u.Texture2D_mut().MipLevels = 1 };

    unsafe {
        d3d12_device.as_ref().unwrap().
        CreateShaderResourceView(
            texture_buffer,
            &shader_resource_view_desc,
            texture_desc_heap.as_ref().unwrap().GetCPUDescriptorHandleForHeapStart()
        )
    };


    let clear_color: [FLOAT; 4] = [ 0.0, 1.0, 1.0, 1.0 ];

    let mut msg = unsafe { mem::MaybeUninit::uninit().assume_init() };

    win::show_window(hwnd);

    loop {
        // quit loop
        if win::quit_window(&mut msg) {
            // report leak
            lib::report_live_objects(d3d12_device, DEBUG);

            return;
        }

        unsafe { TranslateMessage(&mut msg); };
        unsafe { DispatchMessageW(&mut msg); };

        // increment frame
        current_frame += 1;

        // get back buffer index
        let back_buffers_index = unsafe { swapchain.cast::<IDXGISwapChain4>().as_ref().unwrap().GetCurrentBackBufferIndex() };

        // create resource barrier

        let mut barrier_desc = D3D12_RESOURCE_BARRIER {
            Type : D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags : D3D12_RESOURCE_BARRIER_FLAG_NONE,
            u: unsafe { mem::zeroed() },
        };
        * unsafe { barrier_desc.u.Transition_mut() } = D3D12_RESOURCE_TRANSITION_BARRIER {
            pResource : back_buffers[back_buffers_index as usize],
            Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
            StateBefore: D3D12_RESOURCE_STATE_PRESENT,
            StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
        };

        unsafe { cmd_list.as_ref().unwrap().ResourceBarrier(1, &barrier_desc); };

        unsafe { cmd_list.as_ref().unwrap().SetPipelineState(pipeline_state); };

        // set render target
        let mut rtv_heap_start = unsafe { rtv_heaps.as_ref().unwrap().GetCPUDescriptorHandleForHeapStart() };

        rtv_heap_start.ptr += (back_buffers_index * unsafe { d3d12_device.as_ref().unwrap().GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) }) as usize;

        unsafe {
            cmd_list.as_ref().unwrap().OMSetRenderTargets(
                1,
                &rtv_heap_start,
                0,
                std::ptr::null_mut()
            );
        };

        // clear render target
        unsafe {
            cmd_list.as_ref().unwrap().ClearRenderTargetView(rtv_heap_start, &clear_color, 0, std::ptr::null_mut());
        };

        // draw call
        unsafe { cmd_list.as_ref().unwrap().RSSetViewports(1, &viewport); };
        unsafe { cmd_list.as_ref().unwrap().RSSetScissorRects(1, &scissor_rect); };
        unsafe { cmd_list.as_ref().unwrap().SetGraphicsRootSignature(root_signature); };
        unsafe { cmd_list.as_ref().unwrap().IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST); };
        unsafe { cmd_list.as_ref().unwrap().IASetVertexBuffers(0, 1, &vertex_buffer.buffer_view); };
        unsafe { cmd_list.as_ref().unwrap().IASetIndexBuffer(&index_buffer.buffer_view); };
        unsafe { cmd_list.as_ref().unwrap().SetDescriptorHeaps(1, &mut texture_desc_heap); };
		unsafe { cmd_list.as_ref().unwrap().SetGraphicsRootDescriptorTable(0, texture_desc_heap.as_ref().unwrap().GetGPUDescriptorHandleForHeapStart()) };

        unsafe { cmd_list.as_ref().unwrap().DrawIndexedInstanced(indices.len() as u32, 1, 0, 0, 0); };

        // swap barrier state
        unsafe { barrier_desc.u.Transition_mut().StateBefore = D3D12_RESOURCE_STATE_RENDER_TARGET };
        unsafe { barrier_desc.u.Transition_mut().StateAfter = D3D12_RESOURCE_STATE_PRESENT };
        unsafe { cmd_list.as_ref().unwrap().ResourceBarrier(1, &barrier_desc); };

        // run commands
        unsafe { cmd_list.as_ref().unwrap().Close(); };

        let cmd_list_array = [ cmd_list.cast::<ID3D12CommandList>() ];

        unsafe { cmd_queue.as_ref().unwrap().ExecuteCommandLists(1, &cmd_list_array[0]); };

        // handle fence
        unsafe { cmd_queue.as_ref().unwrap().Signal(fence, current_frame); };

        if unsafe { fence.as_ref().unwrap().GetCompletedValue() } != current_frame {
            let event = unsafe { CreateEventW(ptr::null_mut(), 0, 0, ptr::null_mut()) };

            unsafe { fence.as_ref().unwrap().SetEventOnCompletion(current_frame, event); };

            unsafe { WaitForSingleObject(event, INFINITE); };

            unsafe { CloseHandle(event); };
        }

        unsafe { cmd_allocator.as_ref().unwrap().Reset(); };

        unsafe { cmd_list.as_ref().unwrap().Reset(cmd_allocator, pipeline_state); };


        // swap buffer
        unsafe { swapchain.as_ref().unwrap().Present(1, 0); };
    }
}
