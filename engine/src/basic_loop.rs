use crate::core::{
    Loop,
    WGPUContext,
    SurfaceWrapper,
    EventLoopWrapper,
    Application,
};

use winit::{
    dpi::PhysicalSize,
    event::{Event, KeyEvent, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget, ControlFlow},
    keyboard::{Key, NamedKey},
    window::Window,

};

/// A "basic" loop.
pub struct BasicLoop { }

impl Loop for BasicLoop {

    fn init() -> Self {
        BasicLoop {}
    }

    fn start<A: Application>(&self, title: &str, application: A, context: WGPUContext) {

    // init_logger();
      
        let mut application: Option<A> = None;
        let window_loop = EventLoopWrapper::new(title);
        let mut surface = SurfaceWrapper::new();
        // let context = WGPUContext::init_async::<A>(&mut surface, window_loop.window.clone()).await;

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

                target.set_control_flow(ControlFlow::Poll);
                // target.control_flow = ControlFlow::Wait;
                //
                match event {
                    ref e if SurfaceWrapper::start_condition(e) => {
                        surface.resume(&context, window_loop.window.clone(), true); // E::SRGB);

                        // If we haven't created the example yet, do so now.
                        if application.is_none() {
                            application = Some(A::init(
                                // surface.config(),
                                &context,
                                // &context.adapter,
                                // &context.device,
                                // &context.queue,
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
                                // &context.queue,
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
                                .render(&context);

                            frame.present();

                            window_loop.window.request_redraw();
                        }
                        _ => application.as_mut().unwrap().update(&context), // Input cache
                    },
                    _ => {}
                }
            },
        );
    }
}

        // match event {

        //     // Event::NewEvents(start_cause) => {
        //     //     match start_cause {
        //     //         Init => {}
        //     //         _ => {}
        //     //     }
        //     // }

        //     Event::LoopDestroyed => {
        //         application.exit(&device, &queue, &input, &spawner);
        //     }

        //     // TODO: check if pre_update and update are conficting in some circumstances.
        //     Event::MainEventsCleared => {
        //         application.input(&queue, &input);
        //         application.update(&device, &queue, &input, &spawner);
        //         input.pre_update();
        //         window.request_redraw();
        //     }
        //     Event::RedrawEventsCleared => {
        //         #[cfg(not(target_arch = "wasm32"))]
        //         {
        //             spawner.run_until_stalled();
        //         }

        //         let close_application = input.key_state(&Key::Q);
        //         if close_application.is_some() {
        //             *control_flow = ControlFlow::Exit;
        //         }
        //     }
        //     Event::WindowEvent { event, ..} => {
        //         // Update input cache.
        //         input.update(&event);

        //         match event { // Add ScaleFactorChanged.
        //             WindowEvent::Resized(new_size) => {
        //                 size = new_size;
        //                 sc_desc.width = new_size.width.max(1);
        //                 sc_desc.height = new_size.height.max(1);
        //                 surface.configure(&device, &sc_desc);
        //                 application.resize(&device, &sc_desc, size);
        //             }
        //             WindowEvent::CloseRequested => {
        //                 *control_flow = ControlFlow::Exit
        //             }
        //             _ => {}
        //         }
        //     }
        //     Event::RedrawRequested(_) => {
        //         #[cfg(not(target_arch = "wasm32"))]
        //         application.render(&device, &mut queue, &surface, &sc_desc, &spawner);

        //         #[cfg(target_arch = "wasm32")]
        //         application.render(&device, &mut queue, &surface, &sc_desc, &offscreen_canvas_setup, &spawner);
        //     }
        //     _ => { } // Any other events
        // } // match event
    // }); // run
