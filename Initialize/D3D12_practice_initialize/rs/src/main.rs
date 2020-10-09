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
        minwindef::{ UINT, WPARAM, LPARAM, LRESULT, FLOAT },
        winerror::{ S_OK, DXGI_ERROR_NOT_FOUND, SUCCEEDED, FAILED, HRESULT_FROM_WIN32, ERROR_FILE_NOT_FOUND, ERROR_PATH_NOT_FOUND },
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
use std::str;
use std::path::{ Path, PathBuf };
use std::ffi::CString;
use std::env;

const WINDOW_WIDTH: i32 = 1280;
const WINDOW_HEIGHT: i32 = 720;
const DEBUG: bool = true;

type XMFLOAT3 = [f64; 3];
type INDICES = [u32; 6];

enum BOOL {
    FALSE = 0,
    TRUE = 1,
}

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

        // return value 0(HRESULT->S_OK) is ok
        let mut result = -1;

        let mut d3d12_device = ptr::null_mut::<ID3D12Device>();
        let mut dxgi_factory = ptr::null_mut::<IDXGIFactory6>();
        let mut swapchain = ptr::null_mut(); // IDXGISwapChain4
        let mut debug_interface = ptr::null_mut::<ID3D12Debug>();


        if DEBUG {
            result = CreateDXGIFactory2(
                DXGI_CREATE_FACTORY_DEBUG,
                &IID_IDXGIFactory4,
                &mut dxgi_factory as *mut *mut IDXGIFactory6 as *mut *mut c_void,
            );
        } else {
            result = CreateDXGIFactory1(
                &IID_IDXGIFactory1,
                &mut dxgi_factory as *mut *mut IDXGIFactory6 as *mut *mut c_void
            );
        }


        if SUCCEEDED(D3D12GetDebugInterface(
            &IID_ID3D12Debug,
            &mut debug_interface as *mut *mut ID3D12Debug as *mut *mut c_void)) && DEBUG {
            debug_interface.as_ref().unwrap().EnableDebugLayer();
        }


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
                &mut d3d12_device as *mut *mut ID3D12Device as *mut *mut c_void
                )
                 == S_OK {
                feature_level = *lv;
			    break;
            }
        }


        // iterate adapter to use
        let mut tmp_adapter = ptr::null_mut::<IDXGIAdapter>();

        let mut i = 0;

        while  dxgi_factory.as_ref().unwrap().EnumAdapters(i, &mut tmp_adapter as *mut *mut IDXGIAdapter) != DXGI_ERROR_NOT_FOUND {
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

            tmp_adapter.as_ref().unwrap().GetDesc(&mut p_desc);

            if p_desc.Description.to_vec() != encode("NVIDIA") {

                tmp_adapter = tmp_adapter;

                break;
            }

        }

        // create command list, allocator
        let mut cmd_allocator = ptr::null_mut();
        let mut cmd_list = ptr::null_mut::<ID3D12GraphicsCommandList>();

        result = d3d12_device.as_ref().unwrap().CreateCommandAllocator(
            D3D12_COMMAND_LIST_TYPE_DIRECT,
            &IID_ID3D12CommandAllocator,
            &mut cmd_allocator as *mut *mut ID3D12CommandAllocator as *mut *mut c_void
        );

        result = d3d12_device.as_ref().unwrap().CreateCommandList(
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

        result = d3d12_device.as_ref().unwrap().CreateCommandQueue(
            &mut cmd_queue_desc,
            &IID_ID3D12CommandQueue,
            &mut cmd_queue as *mut *mut ID3D12CommandQueue as *mut *mut c_void
        );

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

        // function CreateSwapChainForHwnd 1st argument pDevice is command queue object in IDXGIFactory6
        result = dxgi_factory.as_ref().unwrap().CreateSwapChainForHwnd(
            cmd_queue as *mut IUnknown,
            hwnd,
            &swapchain_desc1,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut swapchain
        );

        // create Render Target View //

        // create discriptor heap
        let mut heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type : D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NodeMask : 0,
            NumDescriptors : 2,
            Flags : D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
        };

        let mut rtv_heaps = std::ptr::null_mut::<ID3D12DescriptorHeap>();

        result = d3d12_device.as_ref().unwrap().CreateDescriptorHeap(
            &mut heap_desc,
            &IID_ID3D12DescriptorHeap,
            &mut rtv_heaps as *mut *mut ID3D12DescriptorHeap as *mut *mut c_void
        );


        // bind render target view heap to swap chain buffer
        let mut back_buffers = vec![std::ptr::null_mut::<ID3D12Resource>(); swapchain_desc1.BufferCount as usize];

        let mut handle = rtv_heaps.as_ref().unwrap().GetCPUDescriptorHandleForHeapStart();

        for i in 0..swapchain_desc1.BufferCount {
            result = swapchain.as_ref().unwrap().GetBuffer(i as u32, &IID_ID3D12Resource, &mut back_buffers[i as usize] as *mut *mut ID3D12Resource as *mut *mut c_void);

            d3d12_device.as_ref().unwrap().CreateRenderTargetView(back_buffers[i as usize], std::ptr::null_mut(), handle);

            handle.ptr += d3d12_device.as_ref().unwrap().GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) as usize;
        }

        // create fence
        let mut fence = std::ptr::null_mut::<ID3D12Fence>();

        result = d3d12_device.as_ref().unwrap().CreateFence(0, D3D12_FENCE_FLAG_NONE, &IID_ID3D12Fence, &mut fence as *mut *mut ID3D12Fence as *mut *mut c_void);


        // create vertices
        let vertices: [XMFLOAT3; 4] = [
            [-0.4, -0.7, 0.0 ],
            [-0.4,  0.7, 0.0 ],
            [ 0.4, -0.7, 0.0 ],
            [ 0.4,  0.7, 0.0 ],
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
            Width : std::mem::size_of::<[XMFLOAT3; 4]>() as u64,
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

        let mut vertex_buffer = std::ptr::null_mut::<ID3D12Resource>();

        result = d3d12_device.as_ref().unwrap().CreateCommittedResource(
            &vertex_buffer_heap_prop,
            D3D12_HEAP_FLAG_NONE,
            &vertex_buffer_resource_desc,
            D3D12_RESOURCE_STATE_GENERIC_READ,
            std::ptr::null_mut(),
            &IID_ID3D12Resource,
            &mut vertex_buffer as *mut *mut ID3D12Resource as *mut *mut c_void
        );

        // vertex buffer map
        let mut vertex_buffer_map = std::ptr::null_mut::<[XMFLOAT3; 4]>();

        result = vertex_buffer.as_ref().unwrap().Map(0, std::ptr::null_mut(), &mut vertex_buffer_map as *mut *mut [XMFLOAT3; 4] as *mut *mut c_void);
        vertex_buffer_map.copy_from_nonoverlapping(&vertices, vertices.len() );
        vertex_buffer.as_ref().unwrap().Unmap(0, std::ptr::null_mut() );


        // create vertex buffer view
        let vertex_buffer_view = D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation : vertex_buffer.as_ref().unwrap().GetGPUVirtualAddress(),
            SizeInBytes : std::mem::size_of::<[XMFLOAT3; 4]>() as u32,
            StrideInBytes : std::mem::size_of::<XMFLOAT3>() as u32,
        };

        // create indices
        let indices: INDICES = [
            0, 1, 2,
            2, 1, 3
        ];

        let mut index_buffer = std::ptr::null_mut::<ID3D12Resource>();
        vertex_buffer_resource_desc.Width = std::mem::size_of::<INDICES>() as u64;

        result = d3d12_device.as_ref().unwrap().CreateCommittedResource(
            &vertex_buffer_heap_prop,
            D3D12_HEAP_FLAG_NONE,
            &vertex_buffer_resource_desc,
            D3D12_RESOURCE_STATE_GENERIC_READ,
            std::ptr::null_mut(),
            &IID_ID3D12Resource,
            &mut index_buffer as *mut *mut ID3D12Resource as *mut *mut c_void
        );

        // indices buffer map
        let mut index_map = std::ptr::null_mut::<INDICES>();
        index_buffer.as_ref().unwrap().Map(0, std::ptr::null_mut(), &mut index_map as *mut *mut INDICES as *mut *mut c_void);
        index_map.copy_from_nonoverlapping(&indices, indices.len() );
        index_buffer.as_ref().unwrap().Unmap(0, std::ptr::null_mut() );

        // create index buffer view
        let index_buffer_view = D3D12_INDEX_BUFFER_VIEW {
            BufferLocation : index_buffer.as_ref().unwrap().GetGPUVirtualAddress(),
            Format : DXGI_FORMAT_R16_UINT,
            SizeInBytes : std::mem::size_of::<INDICES>() as u32,
        };

        // create shader object
        let mut vertex_shader_blob = std::ptr::null_mut::<ID3DBlob>();
        let mut pixel_shader_blob = std::ptr::null_mut::<ID3DBlob>();
        let mut shader_error_blob = std::ptr::null_mut::<ID3DBlob>();

        result = D3DCompileFromFile(
            get_relative_file_path_to_wide_str("shaders\\VertexShader.hlsl").as_ptr() as *const u16,
            std::ptr::null_mut(),
            D3D_COMPILE_STANDARD_FILE_INCLUDE,
            CString::new("BasicVS").unwrap().into_raw(),
            CString::new("vs_5_0").unwrap().into_raw(),
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut vertex_shader_blob,
            &mut shader_error_blob
        );

        result = D3DCompileFromFile(
            get_relative_file_path_to_wide_str("shaders\\PixelShader.hlsl").as_ptr() as *const u16,
            std::ptr::null_mut(),
            D3D_COMPILE_STANDARD_FILE_INCLUDE,
            CString::new("BasicPS").unwrap().into_raw(),
            CString::new("ps_5_0").unwrap().into_raw(),
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION,
            0,
            &mut pixel_shader_blob,
            &mut shader_error_blob
        );


        // notify compilation status
        const FILE_NOT_FOUND: i32 = ERROR_FILE_NOT_FOUND as i32;
        const PATH_NOT_FOUND: i32 = ERROR_PATH_NOT_FOUND as i32;
        match result {
            FILE_NOT_FOUND => println!("{:}", "file not found"),
            PATH_NOT_FOUND => println!("{:}", "path not found"),
            S_OK => println!("{:}", "success compile shader"),
            _ => {
                // TODO
            }
        }

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

        // create graphics pipeline
        let mut gr_pipeline: D3D12_GRAPHICS_PIPELINE_STATE_DESC = mem::zeroed();

        // set shader
        gr_pipeline.pRootSignature = std::ptr::null_mut();
        gr_pipeline.VS.pShaderBytecode = vertex_shader_blob.as_ref().unwrap().GetBufferPointer();
        gr_pipeline.VS.BytecodeLength = vertex_shader_blob.as_ref().unwrap().GetBufferSize();
        gr_pipeline.PS.pShaderBytecode = pixel_shader_blob.as_ref().unwrap().GetBufferPointer();
        gr_pipeline.PS.BytecodeLength = pixel_shader_blob.as_ref().unwrap().GetBufferSize();

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
        let mut render_target_blend_desc: D3D12_RENDER_TARGET_BLEND_DESC = mem::zeroed();
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


        // create root signature
        let mut root_signature = std::ptr::null_mut::<ID3D12RootSignature>();

        let mut root_signature_desc: D3D12_ROOT_SIGNATURE_DESC = mem::zeroed();
        root_signature_desc.Flags = D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT;

        // create root signature binary
        let mut root_signature_blob = std::ptr::null_mut::<ID3DBlob>();

        result = D3D12SerializeRootSignature(
            &root_signature_desc,
            D3D_ROOT_SIGNATURE_VERSION_1_0,
            &mut root_signature_blob,
            &mut shader_error_blob
        );

        result = d3d12_device.as_ref().unwrap().CreateRootSignature(
            0,
            root_signature_blob.as_ref().unwrap().GetBufferPointer(),
            root_signature_blob.as_ref().unwrap().GetBufferSize(),
            &IID_ID3D12RootSignature,
            &mut root_signature as *mut *mut ID3D12RootSignature as *mut *mut c_void
        );

        root_signature_blob.as_ref().unwrap().Release();

        gr_pipeline.pRootSignature = root_signature;

        // create grahphics pipeline state object
        let mut pipeline_state = std::ptr::null_mut::<ID3D12PipelineState>();

        result = d3d12_device.as_ref().unwrap().CreateGraphicsPipelineState(
            &gr_pipeline,
            &IID_ID3D12PipelineState,
            &mut pipeline_state as *mut *mut ID3D12PipelineState as *mut *mut c_void
        );

        // viewport setting
        let mut viewport: D3D12_VIEWPORT = mem::zeroed();
        viewport.Width = WINDOW_WIDTH as f32;
        viewport.Height = WINDOW_HEIGHT as f32;
        viewport.TopLeftX = 0.0;
        viewport.TopLeftY = 0.0;
        viewport.MaxDepth = 1.0;
        viewport.MinDepth = 0.0;

        // scissor rectangle setting
        let mut scissor_rect: D3D12_RECT = mem::zeroed();
        scissor_rect.top = 0;
        scissor_rect.left = 0;
        scissor_rect.right = scissor_rect.left + WINDOW_WIDTH;
        scissor_rect.bottom = scissor_rect.top + WINDOW_HEIGHT;


        // enable debug layer
        // if SUCCEEDED(d3d12_device.as_ref().unwrap().QueryInterface(
        //     &IID_ID3D12Device,
        //     &mut debug_interface as *mut *mut ID3D12DebugDevice as *mut *mut c_void)) && DEBUG {
        //     debug_interface.as_ref().unwrap().ReportLiveDeviceObjects(D3D12_RLDO_DETAIL | D3D12_RLDO_IGNORE_INTERNAL);
        //     debug_interface.as_ref().unwrap().Release();
        // }


        ShowWindow(hwnd, SW_NORMAL);
        UpdateWindow(hwnd);


        let mut current_frame = 0;
        let clear_color: [FLOAT; 4] = [ 1.0, 1.0, 0.0, 1.0 ];

        let mut msg = mem::MaybeUninit::uninit().assume_init();
        loop {
            if GetMessageW(&mut msg, ptr::null_mut(), 0, 0) == 0 {
                return;
            }
            TranslateMessage(&mut msg);
            DispatchMessageW(&mut msg);

            // increment frame
            current_frame += 1;

            // get back buffer index
            let back_buffers_index = swapchain.cast::<IDXGISwapChain4>().as_ref().unwrap().GetCurrentBackBufferIndex();

            // create resource barrier

            let mut barrier_desc = D3D12_RESOURCE_BARRIER {
                Type : D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                Flags : D3D12_RESOURCE_BARRIER_FLAG_NONE,
                u: { mem::zeroed() },
            };
            *{ barrier_desc.u.Transition_mut() } =
                D3D12_RESOURCE_TRANSITION_BARRIER {
                pResource : back_buffers[back_buffers_index as usize],
                Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
            };

            cmd_list.as_ref().unwrap().ResourceBarrier(1, &barrier_desc);

            cmd_list.as_ref().unwrap().SetPipelineState(pipeline_state);

            // set render target
            let mut rtv_heap_start = rtv_heaps.as_ref().unwrap().GetCPUDescriptorHandleForHeapStart();

            rtv_heap_start.ptr += (back_buffers_index * d3d12_device.as_ref().unwrap().GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)) as usize;

            cmd_list.as_ref().unwrap().OMSetRenderTargets(
                1,
                &rtv_heap_start,
                0,
                std::ptr::null_mut()
            );

            // clear render target
            cmd_list.as_ref().unwrap().ClearRenderTargetView(rtv_heap_start, &clear_color, 0, std::ptr::null_mut());

            // draw call
            cmd_list.as_ref().unwrap().RSSetViewports(1, &viewport);
            cmd_list.as_ref().unwrap().RSSetScissorRects(1, &scissor_rect);
            cmd_list.as_ref().unwrap().SetComputeRootSignature(root_signature);
            cmd_list.as_ref().unwrap().IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            cmd_list.as_ref().unwrap().IASetVertexBuffers(0, 1, &vertex_buffer_view);
            cmd_list.as_ref().unwrap().IASetIndexBuffer(&index_buffer_view);
            cmd_list.as_ref().unwrap().DrawIndexedInstanced(indices.len() as u32, 1, 0, 0, 0);

            // swap barrier state
            barrier_desc.u.Transition_mut().StateBefore = D3D12_RESOURCE_STATE_RENDER_TARGET;
            barrier_desc.u.Transition_mut().StateAfter = D3D12_RESOURCE_STATE_PRESENT;
            cmd_list.as_ref().unwrap().ResourceBarrier(1, &barrier_desc);

            // run commands
            cmd_list.as_ref().unwrap().Close();

            let cmd_list_array = [ cmd_list.cast::<ID3D12CommandList>() ];

            cmd_queue.as_ref().unwrap().ExecuteCommandLists(1, &cmd_list_array[0]);

            // handle fence
            cmd_queue.as_ref().unwrap().Signal(fence, current_frame);

            if fence.as_ref().unwrap().GetCompletedValue() != current_frame {
                let event = CreateEventW(ptr::null_mut(), 0, 0, ptr::null_mut());

                fence.as_ref().unwrap().SetEventOnCompletion(current_frame, event);

                WaitForSingleObject(event, INFINITE);

                CloseHandle(event);
            }

            cmd_allocator.as_ref().unwrap().Reset();

            cmd_list.as_ref().unwrap().Reset(cmd_allocator, ptr::null_mut());


            // swap buffer
            swapchain.as_ref().unwrap().Present(1, 0);
        }
    }
}

fn encode(source: &str) -> Vec<u16> {
    source.encode_utf16().chain(Some(0)).collect()
}

fn get_relative_file_path_to_wide_str(s: &str) -> Vec<u16> {
    let relative_path = Path::new(s);
    let pwd = env::current_dir().unwrap();
    let absolute_path = pwd.join(relative_path);
    let wide_str = encode(absolute_path.to_str().unwrap());

    wide_str
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
    }
}
