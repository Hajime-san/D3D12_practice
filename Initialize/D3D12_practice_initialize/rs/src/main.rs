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
};

use std::ptr;
use std::mem;
use std::ffi::CString;

pub mod lib;
pub mod win;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const DEBUG: bool = true;

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

    // bind render target view heap to swap chain buffer
    let back_buffers = lib::create_back_buffer(d3d12_device, swapchain, swapchain_desc1, rtv_heaps, std::ptr::null_mut());

    // create fence
    let fence = lib::create_fence(d3d12_device, 0, D3D12_FENCE_FLAG_NONE).unwrap();

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
        CreationNodeMask: 1,
        VisibleNodeMask: 1,
    };

    // vertex buffer object
    let mut vertex_buffer_resource_desc = D3D12_RESOURCE_DESC {
        Dimension : D3D12_RESOURCE_DIMENSION_BUFFER,
        Alignment: 0,
        Width : (std::mem::size_of_val(&vertices) * 3) as u64,
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

    let committed_resource = lib::CommittedResource {
        pHeapProperties: &vertex_buffer_heap_prop,
        HeapFlags: D3D12_HEAP_FLAG_NONE,
        pResourceDesc: &mut vertex_buffer_resource_desc,
        InitialResourceState: D3D12_RESOURCE_STATE_GENERIC_READ,
        pOptimizedClearValue: std::ptr::null_mut(),
    };

    // create indices
    let indices: lib::INDICES = vec![
        0, 1, 2,
        2, 1, 3
    ];

    // create vertex resources
    let vertex_resources = lib::create_vertex_buffer_view(d3d12_device, committed_resource, vertices.clone(), indices.clone());

    // create shader object
    let shader_error_blob = std::ptr::null_mut::<ID3DBlob>();
    let vertex_shader_blob = lib::create_shader_resource("shaders\\VertexShader.hlsl", "BasicVS", "vs_5_0", shader_error_blob).unwrap();
    let pixel_shader_blob = lib::create_shader_resource("shaders\\PixelShader.hlsl", "BasicPS", "ps_5_0", shader_error_blob).unwrap();

    // vertex layout
    let input_element: [D3D12_INPUT_ELEMENT_DESC; 1] = [
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: CString::new("POSITION").unwrap().into_raw(),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: D3D12_APPEND_ALIGNED_ELEMENT,
            InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
            InstanceDataStepRate: 0,
        }
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
    gr_pipeline.InputLayout.pInputElementDescs = &input_element[0];
    gr_pipeline.InputLayout.NumElements = input_element.len() as u32;

    // way to express triangle
    gr_pipeline.IBStripCutValue = D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED;

    // primitive topology setting
    gr_pipeline.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;

    // render target settings
    gr_pipeline.NumRenderTargets = 1;
    gr_pipeline.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;

    gr_pipeline.RTVFormats[1] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[2] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[3] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[4] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[5] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[6] = DXGI_FORMAT_UNKNOWN;
    gr_pipeline.RTVFormats[7] = DXGI_FORMAT_UNKNOWN;

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

    win::show_window(hwnd);


    let mut current_frame = 0;
    let clear_color: [FLOAT; 4] = [ 1.0, 1.0, 0.0, 1.0 ];

    let mut msg = unsafe { mem::MaybeUninit::uninit().assume_init() };
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
        * unsafe { barrier_desc.u.Transition_mut() } =
            D3D12_RESOURCE_TRANSITION_BARRIER {
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
        unsafe { cmd_list.as_ref().unwrap().IASetVertexBuffers(0, 1, &vertex_resources.vertex_buffer_view); };
        unsafe { cmd_list.as_ref().unwrap().IASetIndexBuffer(&vertex_resources.index_buffer_view); };
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

        unsafe { cmd_list.as_ref().unwrap().Reset(cmd_allocator, ptr::null_mut()); };


        // swap buffer
        unsafe { swapchain.as_ref().unwrap().Present(1, 0); };
    }
}
