// Derived from neovide.
// (C) 2023 Neovide Contributors — licensed under the MIT license.
// See README.md for full license text.

use crate::{File, IntPoint, WindowEvent, int_point};
use skia_safe::{
    ColorSpace, ColorType, Surface,
    gpu::{
        BackendRenderTarget, DirectContext, FlushInfo, Protected, SurfaceOrigin, SyncCpu,
        d3d::{BackendContext, ID3D12CommandQueue, ID3D12Resource, TextureResourceInfo},
        surfaces::wrap_backend_render_target,
    },
    surface::BackendSurfaceAccess,
};
use std::{
    ffi::{OsString, c_void},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HGLOBAL, HWND},
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_11_0,
            Direct3D12::{
                D3D12_COMMAND_LIST_TYPE_DIRECT, D3D12_COMMAND_QUEUE_DESC,
                D3D12_COMMAND_QUEUE_FLAG_NONE, D3D12_FENCE_FLAG_NONE, D3D12_RESOURCE_STATE_PRESENT,
                D3D12CreateDevice, ID3D12Device, ID3D12Fence,
            },
            DirectComposition::{
                DCompositionCreateDevice2, IDCompositionDevice, IDCompositionTarget,
                IDCompositionVisual,
            },
            Dxgi::{
                Common::{
                    DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_FORMAT_UNKNOWN,
                    DXGI_SAMPLE_DESC,
                },
                CreateDXGIFactory1, DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_SOFTWARE, DXGI_PRESENT,
                DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_CHAIN_FLAG,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
                DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGIAdapter1,
                IDXGIFactory2, IDXGISwapChain1, IDXGISwapChain3,
            },
        },
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
                OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
            Ole::{CF_HDROP, CF_WAVE},
            Threading::{CreateEventW, INFINITE, WaitForSingleObjectEx},
        },
        UI::{
            HiDpi::GetDpiForWindow,
            Shell::{DragQueryFileW, HDROP},
        },
    },
    core::{Interface, PCWSTR, Result, w},
};

use super::{Clipboard, PlatformCommon};

pub struct WindowsClipboard {
    hwnd: HWND,
}

const CLIPBOARD_FORMAT: windows::core::PCWSTR = w!("com.httnn.tahti_clipboard.v1");

struct ClipboardGuard;

impl ClipboardGuard {
    fn new(hwnd: HWND) -> Option<Self> {
        unsafe { OpenClipboard(Some(hwnd)).ok()? };
        Some(Self)
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

impl Clipboard for WindowsClipboard {
    fn write_string(&mut self, text: impl Into<String>) {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard.set_text(text.into()).unwrap();
    }

    fn write_data(&mut self, data: Vec<u8>) {
        unsafe {
            let clipboard_guard = ClipboardGuard::new(self.hwnd);
            if clipboard_guard.is_none() {
                return;
            }

            let format = RegisterClipboardFormatW(CLIPBOARD_FORMAT);
            if format == 0 {
                return;
            }

            let _ = EmptyClipboard();

            let size = data.len();
            if let Ok(hmem) = GlobalAlloc(GMEM_MOVEABLE, size) {
                if hmem.is_invalid() {
                    return;
                }

                let ptr = GlobalLock(hmem);
                if !ptr.is_null() {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, size);
                    let _ = GlobalUnlock(hmem);
                    let _ = SetClipboardData(format, Some(HANDLE(hmem.0)));
                }
            }
        }
    }

    fn read_data(&self) -> Option<Vec<u8>> {
        unsafe {
            let clipboard_guard = ClipboardGuard::new(self.hwnd);
            if clipboard_guard.is_none() {
                return None;
            }

            let format = RegisterClipboardFormatW(CLIPBOARD_FORMAT);

            if format != 0 && IsClipboardFormatAvailable(format).is_ok() {
                let handle = GetClipboardData(format).ok()?;
                let hmem = HGLOBAL(handle.0);

                if hmem.is_invalid() {
                    None
                } else {
                    let ptr = GlobalLock(hmem);
                    if ptr.is_null() {
                        None
                    } else {
                        let size = GlobalSize(hmem);
                        let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                        let vec = slice.to_vec();
                        let _ = GlobalUnlock(hmem);
                        Some(vec)
                    }
                }
            } else {
                None
            }
        }
    }

    fn read_string(&self) -> Option<String> {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard.get_text().ok()
    }

    fn read_paths(&self) -> Option<Vec<PathBuf>> {
        unsafe {
            if !OpenClipboard(Some(self.hwnd)).is_ok() {
                return None;
            }

            let mut out = Vec::new();
            if IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() {
                let hdrop = GetClipboardData(CF_HDROP.0 as u32).unwrap();
                if !hdrop.is_invalid() {
                    // count files
                    let count = DragQueryFileW(HDROP(hdrop.0), u32::MAX, None);
                    // buffer for each path
                    for i in 0..count {
                        let mut buf = [0u16; 260]; // MAX_PATH
                        let len = DragQueryFileW(HDROP(hdrop.0), i, Some(&mut buf));
                        if len > 0 {
                            let os_str = OsString::from_wide(&buf[..len as usize]);
                            out.push(PathBuf::from(os_str));
                        }
                    }
                }
            }
            let _ = CloseClipboard();

            if out.is_empty() { None } else { Some(out) }
        }
    }

    fn read_audio(&self) -> Option<Vec<u8>> {
        unsafe {
            if !OpenClipboard(Some(self.hwnd)).is_ok() {
                return None;
            }

            let data = if IsClipboardFormatAvailable(CF_WAVE.0 as u32).is_ok() {
                let hmem = HGLOBAL(GetClipboardData(CF_WAVE.0 as u32).unwrap().0);
                if !hmem.is_invalid() {
                    let ptr = GlobalLock(hmem);
                    if !ptr.is_null() {
                        let size = GlobalSize(hmem);
                        let slice = std::slice::from_raw_parts(ptr as *const u8, size);
                        let vec = slice.to_vec();
                        let _ = GlobalUnlock(hmem);
                        Some(vec)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let _ = CloseClipboard();
            data
        }
    }
}

fn get_hardware_adapter(factory: &IDXGIFactory2) -> Result<IDXGIAdapter1> {
    for i in 0.. {
        let adapter = unsafe { factory.EnumAdapters1(i)? };
        let desc = unsafe { adapter.GetDesc1() }?;

        if DXGI_ADAPTER_FLAG(desc.Flags as i32).contains(DXGI_ADAPTER_FLAG_SOFTWARE) {
            continue;
        }

        unsafe {
            if D3D12CreateDevice(
                &adapter,
                D3D_FEATURE_LEVEL_11_0,
                &mut Option::<ID3D12Device>::None,
            )
            .is_ok()
            {
                return Ok(adapter);
            }
        }
    }

    // As this function returns `Ok()` when successfully enumerated all of adapters
    // or `Err()` when failed, this code will never reach here.
    unreachable!()
}

pub struct Frame {
    surface: Surface,
    surface_index: usize,
}

impl super::Frame for Frame {
    fn canvas(&mut self) -> crate::Canvas<'_> {
        crate::Canvas::new(self.surface.canvas())
    }
}

const BUFFER_COUNT: u32 = 2;

pub struct Platform {
    hwnd: HWND,
    gr_context: DirectContext,
    swap_chain: IDXGISwapChain3,
    swap_chain_desc: DXGI_SWAP_CHAIN_DESC1,
    swap_chain_waitable: HANDLE,
    pub command_queue: ID3D12CommandQueue,
    buffers: Vec<ID3D12Resource>,
    surfaces: Vec<Option<Surface>>,
    fence_values: Vec<u64>,
    fence: ID3D12Fence,
    fence_event: HANDLE,
    frame_swapped: bool,
    frame_index: usize,
    size: IntPoint,
    clipboard: WindowsClipboard,
    _backend_context: BackendContext,
    #[cfg(feature = "gpu_profiling")]
    pub device: ID3D12Device,
    _adapter: IDXGIAdapter1,
    _composition_device: IDCompositionDevice,
    _target: IDCompositionTarget,
    _visual: IDCompositionVisual,
}

unsafe impl Send for Platform {}

impl PlatformCommon for Platform {
    fn documents_folder_path() -> Option<PathBuf> {
        None
    }

    fn open_url(&self, url: impl Into<String>) {
        let _ = open::that(url.into());
    }

    fn next_window_event(&mut self) -> Option<WindowEvent> {
        None
    }

    fn start_drag(&self, _path: std::path::PathBuf) {}

    fn get_scale_factor(&self) -> f32 {
        if self.hwnd.0 != std::ptr::null_mut() {
            let dpi = unsafe { GetDpiForWindow(self.hwnd) };
            dpi as f32 / 96.0
        } else {
            1.0
        }
    }

    fn set_view_size(&mut self, size: (u32, u32)) {
        let scale = self.get_scale_factor();
        let width = (size.0 as f32 * scale) as u32;
        let height = (size.1 as f32 * scale) as u32;
        self.size = (width, height).into();

        // Clean up any outstanding resources in command lists
        self.gr_context.flush_submit_and_sync_cpu();

        self.wait_for_gpu();

        self.surfaces.clear();
        self.buffers.clear();

        unsafe {
            self.swap_chain
                .ResizeBuffers(
                    0,
                    width,
                    height,
                    DXGI_FORMAT_UNKNOWN,
                    DXGI_SWAP_CHAIN_FLAG(self.swap_chain_desc.Flags as i32),
                )
                .expect("Failed to resize buffers");
        }
        self.setup_surfaces();
    }

    fn clipboard(&mut self) -> &mut impl Clipboard {
        &mut self.clipboard
    }

    fn open_path_in_file_explorer(&self, path: PathBuf) {
        std::process::Command::new("explorer")
            .arg("/select,")
            .arg(path.to_path_buf().into_os_string())
            .spawn()
            .unwrap();
    }

    fn file_open_dialog(&self, opts: super::FileOpenOptions) {
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(opts.filetype_desc, &opts.extensions)
                .set_directory("~")
                .pick_file()
            {
                opts.result.set(vec![File::Path(path)]);
            }
        });
    }

    fn file_save_dialog(&self, options: super::FileSaveOptions) {
        std::thread::spawn(move || {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name(options.filename)
                .add_filter(options.filetype_desc, &[options.extension])
                .set_directory("~")
                .save_file()
            {
                std::fs::write(path.clone(), (*options.data).clone()).unwrap();
            }
        });
    }

    fn open_prompt(
        &self,
        _title: String,
        _enter_text: String,
        _value: String,
        _input_type: super::InputType,
        _result: &crate::Later<String>,
    ) {
    }

    type Frame = Frame;

    fn new_frame(&mut self) -> Option<Self::Frame> {
        // Only block the cpu when whe actually need to draw to the canvas
        if self.frame_swapped {
            self.move_to_next_frame();
        }
        if let Some(mut surface) = self.surfaces[self.frame_index].take() {
            surface.canvas().save();
            Some(Frame {
                surface,
                surface_index: self.frame_index,
            })
        } else {
            None
        }
    }

    fn end_frame(&mut self, mut frame: Self::Frame) {
        frame.surface.canvas().restore();
        self.surfaces[frame.surface_index] = Some(frame.surface);
        // {
        //     tracy_gpu_zone!("wait for vsync");
        //     vsync.wait_for_vsync();
        // }
        self.swap_buffers();
    }
}

impl Platform {
    pub fn new_from_window_handle(hwnd: *mut c_void) -> Self {
        let hwnd = HWND(hwnd as *mut _);
        #[cfg(feature = "d3d_debug")]
        let dxgi_factory: IDXGIFactory2 = unsafe {
            let mut debug_controller: Option<ID3D12Debug> = None;
            D3D12GetDebugInterface(&mut debug_controller)
                .expect("Failed to create Direct3D debug controller");

            debug_controller
                .expect("Failed to enable debug layer")
                .EnableDebugLayer();

            CreateDXGIFactory2(DXGI_CREATE_FACTORY_DEBUG).expect("Failed to create DXGI factory")
        };

        #[cfg(not(feature = "d3d_debug"))]
        let dxgi_factory: IDXGIFactory2 =
            unsafe { CreateDXGIFactory1().expect("Failed to create DXGI factory") };

        let adapter = get_hardware_adapter(&dxgi_factory)
            .expect("Failed to find any suitable Direct3D 12 adapters");

        let mut device: Option<ID3D12Device> = None;
        unsafe {
            D3D12CreateDevice(&adapter, D3D_FEATURE_LEVEL_11_0, &mut device)
                .expect("Failed to create a Direct3D 12 device");
        }
        let device = device.expect("Failed to create a Direct3D 12 device");

        // Describe and create the command queue.
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
            ..Default::default()
        };
        let command_queue: ID3D12CommandQueue = unsafe {
            device
                .CreateCommandQueue(&queue_desc)
                .expect("Failed to create the Direct3D command queue")
        };

        let size = int_point(1000, 1000);

        // Describe and create the swap chain.
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            Width: size.x as u32, // TODO: uhhh
            Height: size.y as u32,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            Stereo: false.into(),
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: DXGI_SCALING_STRETCH,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
            Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
        };

        let swap_chain = unsafe {
            dxgi_factory
                .CreateSwapChainForComposition(&command_queue, &swap_chain_desc, None)
                .expect("Failed to create the Direct3D swap chain")
        };

        let swap_chain: IDXGISwapChain3 =
            IDXGISwapChain1::cast(&swap_chain).expect("Failed to cast");

        unsafe {
            swap_chain
                .SetMaximumFrameLatency(1)
                .expect("Failed to set maximum frame latency");
        }
        let composition_device: IDCompositionDevice = unsafe {
            DCompositionCreateDevice2(None).expect("Could not create composition device")
        };
        let target = unsafe {
            composition_device
                .CreateTargetForHwnd(hwnd, true)
                .expect("Could not create composition target")
        };
        let visual = unsafe {
            composition_device
                .CreateVisual()
                .expect("Could not create composition visual")
        };

        unsafe {
            visual
                .SetContent(&swap_chain)
                .expect("Failed to set composition content");
            target
                .SetRoot(&visual)
                .expect("Failed to set composition root");
            composition_device
                .Commit()
                .expect("Failed to commit composition");
        }

        let swap_chain_waitable = unsafe { swap_chain.GetFrameLatencyWaitableObject() };
        if swap_chain_waitable.is_invalid() {
            panic!("Failed to get swapchain waitable object");
        }

        // use a high value to make it easier to track these in PIX
        let fence_values = vec![10000; swap_chain_desc.BufferCount as usize];
        let fence: ID3D12Fence = unsafe {
            device
                .CreateFence(fence_values[0], D3D12_FENCE_FLAG_NONE)
                .expect("Failed to create fence")
        };

        let fence_event = unsafe {
            CreateEventW(None, false, false, PCWSTR::null()).expect("Failed to create event")
        };
        let frame_index = unsafe { swap_chain.GetCurrentBackBufferIndex() as usize };

        let backend_context = BackendContext {
            adapter: adapter.clone(),
            device: device.clone(),
            queue: command_queue.clone(),
            memory_allocator: None,
            protected_context: Protected::No,
        };
        let gr_context = unsafe {
            DirectContext::new_d3d(&backend_context, None).expect("Failed to create Skia context")
        };

        let mut ret = Self {
            hwnd,
            _adapter: adapter,
            #[cfg(feature = "gpu_profiling")]
            device,
            command_queue,
            swap_chain,
            swap_chain_desc,
            swap_chain_waitable,
            gr_context,
            _backend_context: backend_context,
            buffers: Vec::new(),
            surfaces: Vec::new(),
            fence_values,
            fence,
            fence_event,
            frame_swapped: true,
            frame_index,
            size,
            clipboard: WindowsClipboard { hwnd: hwnd },
            _composition_device: composition_device,
            _target: target,
            _visual: visual,
        };
        ret.setup_surfaces();

        ret
    }

    fn setup_surfaces(&mut self) {
        let size = (
            self.size.x.try_into().expect("Could not convert width"),
            self.size.y.try_into().expect("Could not convert height"),
        );

        self.buffers.clear();
        self.surfaces.clear();
        for i in 0..self.swap_chain_desc.BufferCount {
            let buffer: ID3D12Resource = unsafe {
                self.swap_chain
                    .GetBuffer(i)
                    .expect("Could not get swapchain buffer")
            };
            self.buffers.push(buffer.clone());

            let info = TextureResourceInfo {
                resource: buffer,
                alloc: None,
                resource_state: D3D12_RESOURCE_STATE_PRESENT,
                format: self.swap_chain_desc.Format,
                sample_count: self.swap_chain_desc.SampleDesc.Count,
                level_count: 1,
                sample_quality_pattern: 0,
                protected: Protected::No,
            };

            let surface = wrap_backend_render_target(
                &mut self.gr_context,
                &BackendRenderTarget::new_d3d(size, &info),
                SurfaceOrigin::TopLeft,
                ColorType::RGBA8888,
                ColorSpace::new_srgb(),
                None,
            )
            .expect("Could not create backend render target");
            self.surfaces.push(Some(surface));
        }
        self.frame_index = unsafe { self.swap_chain.GetCurrentBackBufferIndex() as usize };
    }

    fn wait_for_gpu(&mut self) {
        unsafe {
            let current_fence_value = *self.fence_values.iter().max().unwrap();
            // Schedule a Signal command in the queue.
            self.command_queue
                .Signal(&self.fence, current_fence_value)
                .unwrap();

            // Wait until the fence has been processed.
            self.fence
                .SetEventOnCompletion(current_fence_value, self.fence_event)
                .unwrap();
            WaitForSingleObjectEx(self.fence_event, INFINITE, false);

            // Increment all fence values
            for v in &mut self.fence_values {
                *v = current_fence_value + 1;
            }
        }
    }

    fn move_to_next_frame(&mut self) {
        if self.frame_swapped {
            unsafe {
                let current_fence_value = self.fence_values[self.frame_index];

                // Schedule a Signal command in the queue.
                self.command_queue
                    .Signal(&self.fence, current_fence_value)
                    .unwrap();

                // Update the frame index.
                self.frame_index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
                let old_fence_value = self.fence_values[self.frame_index];

                // If the next frame is not ready to be rendered yet, wait until it is ready.
                if self.fence.GetCompletedValue() < old_fence_value {
                    self.fence
                        .SetEventOnCompletion(old_fence_value, self.fence_event)
                        .unwrap();
                    WaitForSingleObjectEx(self.fence_event, INFINITE, false);
                }

                // Set the fence value for the next frame.
                self.fence_values[self.frame_index] = current_fence_value + 1;
                self.frame_swapped = false;
            }
        }
    }

    fn swap_buffers(&mut self) {
        unsafe {
            // Switch the back buffer resource state to present For some reason the
            // DirectContext::flush_and_submit does not do that for us automatically.
            let buffer_index = self.swap_chain.GetCurrentBackBufferIndex() as usize;
            if let Some(surface) = &mut self.surfaces[buffer_index] {
                self.gr_context.flush_surface_with_access(
                    surface,
                    BackendSurfaceAccess::Present,
                    &FlushInfo::default(),
                );
                self.gr_context.submit(Some(SyncCpu::No));

                if self.swap_chain.Present(1, DXGI_PRESENT(0)).is_ok() {
                    self.frame_swapped = true;
                }
            }
        }
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            self.gr_context.release_resources_and_abandon();
            self.wait_for_gpu();
            CloseHandle(self.fence_event).unwrap();
        }
    }
}
