use wgpu::TextureView;
use crate::input_cache::InputCache;
use std::sync::Arc;
use winit::{
    dpi::PhysicalSize,
    event::{Event,StartCause}, // KeyEvent, , WindowEvent},
    event_loop::{EventLoop}, //, EventLoopWindowTarget},
    // keyboard::{Key, NamedKey},
    window::Window,
};

use wgpu::{Instance /*, Surface */};

/// Context containing global wgpu resources.
pub struct WGPUContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}


/// A trait for event loops.
pub trait Loop: Sized + 'static {
    fn init() -> Self;
    fn start<A: Application>(title: &str, context: WGPUContext, surface: SurfaceWrapper, window_loop: EventLoopWrapper); //, wgpu_context: WGPUContext);
}

/// A trait for application wgpu-features.
pub trait WGPUFeatures: Sized + 'static {

    // const SRGB: bool = true;

    fn optional_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_features() -> wgpu::Features {
        wgpu::Features::empty()
    }

    fn required_downlevel_capabilities() -> wgpu::DownlevelCapabilities {
        wgpu::DownlevelCapabilities {
            flags: wgpu::DownlevelFlags::empty(),
            shader_model: wgpu::ShaderModel::Sm5,
            ..wgpu::DownlevelCapabilities::default()
        }
    }

    fn required_limits() -> wgpu::Limits {
        wgpu::Limits::downlevel_webgl2_defaults()
    }
}

pub struct EventLoopWrapper {
     pub event_loop: EventLoop<()>,
     pub window: Arc<Window>,
 }
 
 impl EventLoopWrapper {
     pub fn new(title: &str) -> Self {
         let event_loop = EventLoop::new().unwrap();
         let mut builder = winit::window::WindowBuilder::new();
         #[cfg(target_arch = "wasm32")]
         {
             use wasm_bindgen::JsCast;
             use winit::platform::web::WindowBuilderExtWebSys;
             let canvas = web_sys::window()
                 .unwrap()
                 .document()
                 .unwrap()
                 .get_element_by_id("canvas")
                 .unwrap()
                 .dyn_into::<web_sys::HtmlCanvasElement>()
                 .unwrap();
             builder = builder.with_canvas(Some(canvas));
         }
         builder = builder.with_title(title);
         let window = Arc::new(builder.build(&event_loop).unwrap());
 
         Self { event_loop, window }
     }
 }
 
pub struct SurfaceWrapper {
     pub surface: Option<wgpu::Surface<'static>>,
     config: Option<wgpu::SurfaceConfiguration>,
 }
 
 impl SurfaceWrapper {
     /// Create a new surface wrapper with no surface or configuration.
     pub fn new() -> Self {
         Self {
             surface: None,
             config: None,
         }
     }
 
     /// Called after the instance is created, but before we request an adapter.
     ///
     /// On wasm, we need to create the surface here, as the WebGL backend needs
     /// a surface (and hence a canvas) to be present to create the adapter.
     ///
     /// We cannot unconditionally create a surface here, as Android requires
     /// us to wait until we recieve the `Resumed` event to do so.
     pub fn pre_adapter(&mut self, instance: &Instance, window: Arc<Window>) {
         if cfg!(target_arch = "wasm32") {
             self.surface = Some(instance.create_surface(window).unwrap());
         }
     }
 
     /// Check if the event is the start condition for the surface.
     pub fn start_condition(e: &Event<()>) -> bool {
         match e {
             // On all other platforms, we can create the surface immediately.
             Event::NewEvents(StartCause::Init) => !cfg!(target_os = "android"),
             // On android we need to wait for a resumed event to create the surface.
             Event::Resumed => cfg!(target_os = "android"),
             _ => false,
         }
     }
 
     /// Called when an event which matches [`Self::start_condition`] is recieved.
     ///
     /// On all native platforms, this is where we create the surface.
     ///
     /// Additionally, we configure the surface based on the (now valid) window size.
     pub fn resume(&mut self, context: &WGPUContext, window: Arc<Window>, srgb: bool) {
         // Window size is only actually valid after we enter the event loop.
         let window_size = window.inner_size();
         let width = window_size.width.max(1);
         let height = window_size.height.max(1);
 
         log::info!("Surface resume {window_size:?}");
 
         // We didn't create the surface in pre_adapter, so we need to do so now.
         if !cfg!(target_arch = "wasm32") {
             self.surface = Some(context.instance.create_surface(window).unwrap());
         }
 
         // From here on, self.surface should be Some.
 
         let surface = self.surface.as_ref().unwrap();
 
         // Get the default configuration,
         let mut config = surface
             .get_default_config(&context.adapter, width, height)
             .expect("Surface isn't supported by the adapter.");
         if srgb {
             // Not all platforms (WebGPU) support sRGB swapchains, so we need to use view formats
             let view_format = config.format.add_srgb_suffix();
             config.view_formats.push(view_format);
         } else {
             // All platforms support non-sRGB swapchains, so we can just use the format directly.
             let format = config.format.remove_srgb_suffix();
             config.format = format;
             config.view_formats.push(format);
         };
 
         surface.configure(&context.device, &config);
         self.config = Some(config);
     }
 
     /// Resize the surface, making sure to not resize to zero.
     pub fn resize(&mut self, context: &WGPUContext, size: PhysicalSize<u32>) {
         log::info!("Surface resize {size:?}");
 
         let config = self.config.as_mut().unwrap();
         config.width = size.width.max(1);
         config.height = size.height.max(1);
         let surface = self.surface.as_ref().unwrap();
         surface.configure(&context.device, config);
     }
 
     /// Acquire the next surface texture.
     pub fn acquire(&mut self, context: &WGPUContext) -> wgpu::SurfaceTexture {
         let surface = self.surface.as_ref().unwrap();
 
         match surface.get_current_texture() {
             Ok(frame) => frame,
             // If we timed out, just try again
             Err(wgpu::SurfaceError::Timeout) => surface
                 .get_current_texture()
                 .expect("Failed to acquire next surface texture!"),
             Err(
                 // If the surface is outdated, or was lost, reconfigure it.
                 wgpu::SurfaceError::Outdated
                 | wgpu::SurfaceError::Lost
                 // If OutOfMemory happens, reconfiguring may not help, but we might as well try
                 | wgpu::SurfaceError::OutOfMemory,
             ) => {
                 surface.configure(&context.device, self.config());
                 surface
                     .get_current_texture()
                     .expect("Failed to acquire next surface texture!")
             }
         }
     }
 
     /// On suspend on android, we drop the surface, as it's no longer valid.
     ///
     /// A suspend event is always followed by at least one resume event.
     pub fn suspend(&mut self) {
         if cfg!(target_os = "android") {
             self.surface = None;
         }
     }
 
     pub fn get(&self) -> Option<&wgpu::Surface> {
         self.surface.as_ref()
     }
 
     pub fn config(&self) -> &wgpu::SurfaceConfiguration {
         self.config.as_ref().unwrap()
     }
 }
 
 pub trait Application: Sized + 'static {
 
     /// Initialization of the application.
     fn init(wgpu_context: &WGPUContext, surface: &SurfaceWrapper) -> Self; //, input_cache: &InputCache);
                                                       
     /// Rendering of the application.
     fn render(&mut self, context: &WGPUContext, view: &TextureView, surface: &SurfaceWrapper);
 
     /// Resizing of the application.
     fn resize(&mut self, wgpu_context: &WGPUContext, surface_configuration: &wgpu::SurfaceConfiguration, new_size: winit::dpi::PhysicalSize<u32>);
 
     /// Updating of the application.
     fn update(&mut self, wgpu_context: &WGPUContext, input_cache: &InputCache);
                                                         
     /// Closing of the application.
     fn close(&mut self, wgpu_context: &WGPUContext);
 }
 
/// Create wgpu core elements for application.
pub async fn setup<F: WGPUFeatures>(surface: &mut SurfaceWrapper, window: Arc<Window>) -> WGPUContext {
    log::info!("Initializing wgpu...");

    let backends = wgpu::util::backend_bits_from_env().unwrap_or_default();
    let dx12_shader_compiler = wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default();
    let gles_minor_version = wgpu::util::gles_minor_version_from_env().unwrap_or_default();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        dx12_shader_compiler,
        gles_minor_version,
    });
    surface.pre_adapter(&instance, window);
    let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, surface.get())
        .await
        .expect("No suitable GPU adapters found on the system!");

    let adapter_info = adapter.get_info();
    log::info!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

    let optional_features = F::optional_features();
    let required_features = F::required_features();
    let adapter_features = adapter.features();
    assert!(
        adapter_features.contains(required_features),
        "Adapter does not support required features for this example: {:?}",
        required_features - adapter_features
        );

    let required_downlevel_capabilities = F::required_downlevel_capabilities();
    let downlevel_capabilities = adapter.get_downlevel_capabilities();
    assert!(
        downlevel_capabilities.shader_model >= required_downlevel_capabilities.shader_model,
        "Adapter does not support the minimum shader model required to run this example: {:?}",
        required_downlevel_capabilities.shader_model
        );
    assert!(
        downlevel_capabilities
        .flags
        .contains(required_downlevel_capabilities.flags),
        "Adapter does not support the downlevel capabilities required to run this example: {:?}",
        required_downlevel_capabilities.flags - downlevel_capabilities.flags
        );

    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the surface.
    let needed_limits = F::required_limits().using_resolution(adapter.limits());

    let trace_dir = std::env::var("WGPU_TRACE");
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: (optional_features & adapter_features) | required_features,
                required_limits: needed_limits,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            trace_dir.ok().as_ref().map(std::path::Path::new),
            )
        .await
        .expect("Unable to find a suitable GPU adapter!");

    WGPUContext {
        instance,
        adapter,
        device,
        queue,
    }
}

pub fn run<F: WGPUFeatures, L: Loop, A: Application>(title: &'static str) {

    // Create surface.

    log::info!("Creating surface wrapper...");
    let mut surface = SurfaceWrapper::new();

    // Event loop.
    log::info!("Creating eventloop wrapper...");
    let window_loop = EventLoopWrapper::new(title);

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            wasm_bindgen_futures::spawn_local(async move {

                log::info!("Creating WGPUContext...");
                let context = setup::<F>(&mut surface, window_loop.window.clone()).await.unwrap();

                log::info!("Staring event loop...");
                L::start::<A>(&self, title: &str, context, surface, window_loop);
            })
        } else {

            log::info!("Creating WGPUContext...");
            let context = pollster::block_on(setup::<F>(&mut surface, window_loop.window.clone()));

            log::info!("Staring event loop...");
            L::start::<A>(title, context, surface, window_loop);
        }
    }
}

