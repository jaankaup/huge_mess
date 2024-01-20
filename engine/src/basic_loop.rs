use crate::input_cache::InputCache;
use crate::core::{
    Loop,
    WGPUContext,
    SurfaceWrapper,
    EventLoopWrapper,
    Application,
};

use winit::{
    event::{Event, KeyEvent, /* StartCause, */ WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget}, //, ControlFlow},
    keyboard::{Key, NamedKey},
};

/// A "basic" loop.
pub struct BasicLoop { }

impl Loop for BasicLoop {

    fn init() -> Self {
        BasicLoop {}
    }

    // fn start<A: Application>(&self, title: &str, context: WGPUContext, surface: SurfaceWrapper) {
    fn start<A: Application>(_title: &str, context: WGPUContext, mut surface: SurfaceWrapper, window_loop: EventLoopWrapper) {

        let mut application = None;
        let mut input_cache = InputCache::init();

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                use winit::platform::web::EventLoopExtWebSys;
                let event_loop_function = EventLoop::spawn;
            } else {
                let event_loop_function = EventLoop::run;
            }
        }

        log::info!("Entering event loop...");

        // On native this is a result, but on wasm it's a unit type.
        #[allow(clippy::let_unit_value)]
        let _ = (event_loop_function)(
            window_loop.event_loop,
            move |event: Event<()>, target: &EventLoopWindowTarget<()>| {

                // target.set_control_flow(ControlFlow::Poll);
                // target.set_control_flow(ControlFlow::Wait);

                match event {
                    ref e if SurfaceWrapper::start_condition(e) => {
                        surface.resume(&context, window_loop.window.clone(), true); // E::SRGB);

                        // If we haven't created the example yet, do so now.
                        if application.is_none() {
                            application = Some(A::init(
                                &context,
                            ));
                        }
                    }
                    Event::Suspended => {
                        surface.suspend();
                    }
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::Resized(size) => {
                            surface.resize(&context, size);
                            application.as_mut().unwrap().resize(
                                &context,
                                surface.config(),
                                size
                            );

                            window_loop.window.request_redraw();
                        }
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Named(NamedKey::Escape),
                                    ..
                                },
                            ..
                        }
                        | WindowEvent::CloseRequested => {
                            target.exit();
                        }
                        #[cfg(not(target_arch = "wasm32"))]
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    logical_key: Key::Character(s),
                                    ..
                                },
                            ..
                        } if s == "r" => {
                            println!("{:#?}", context.instance.generate_report());
                        }
                        WindowEvent::RedrawRequested => {
                            // On MacOS, currently redraw requested comes in _before_ Init does.
                            // If this happens, just drop the requested redraw on the floor.
                            //
                            // See https://github.com/rust-windowing/winit/issues/3235 for some discussion
                            if application.is_none() {
                                return;
                            }

                            // frame_counter.update();

                            let frame = surface.acquire(&context);
                            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                                format: Some(surface.config().view_formats[0]),
                                ..wgpu::TextureViewDescriptor::default()
                            });

                            application
                                .as_mut()
                                .unwrap()
                                .render(&context, &view, &surface);

                            frame.present();

                            window_loop.window.request_redraw();
                        }
                        _ => {
                            input_cache.update(&event);
                            application.as_mut().unwrap().update(&context, &input_cache ); // Input cache
                        },
                    },
                    _ => {}
                }
            },
        );
    }
}
