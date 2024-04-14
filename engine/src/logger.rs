// use simple_logger::SimpleLogger;
use log::LevelFilter;

// fn init_logger() {
//     cfg_if::cfg_if! {
//         if #[cfg(target_arch = "wasm32")] {
//             // As we don't have an environment to pull logging level from, we use the query string.
//             let query_string = web_sys::window().unwrap().location().search().unwrap();
//             let query_level: Option<log::LevelFilter> = parse_url_query_string(&query_string, "RUST_LOG")
//                 .and_then(|x| x.parse().ok());
// 
//             // We keep wgpu at Error level, as it's very noisy.
//             let base_level = query_level.unwrap_or(log::LevelFilter::Info);
//             let wgpu_level = query_level.unwrap_or(log::LevelFilter::Error);
// 
//             // On web, we use fern, as console_log doesn't have filtering on a per-module level.
//             fern::Dispatch::new()
//                 .level(base_level)
//                 .level_for("wgpu_core", wgpu_level)
//                 .level_for("wgpu_hal", wgpu_level)
//                 .level_for("naga", wgpu_level)
//                 .chain(fern::Output::call(console_log::log))
//                 .apply()
//                 .unwrap();
//             std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//         } else {
//             // parse_default_env will read the RUST_LOG environment variable and apply it on top
//             // of these default filters.
//             env_logger::builder()
//                 .filter_level(log::LevelFilter::Info)
//                 // We keep wgpu at Error level, as it's very noisy.
//                 .filter_module("wgpu_core", log::LevelFilter::Info)
//                 .filter_module("wgpu_hal", log::LevelFilter::Error)
//                 .filter_module("naga", log::LevelFilter::Error)
//                 .parse_default_env()
//                 .init();
//         }
//     }
// }

// /// Initialize simple logger.
// pub fn initialize_simple_logger(module_levels: &Vec<(String, LevelFilter)>) {
// 
//     #[cfg(not(target_arch = "wasm32"))]
//     {
// 
//         let mut simple_logger = SimpleLogger::new().with_level(LevelFilter::Off);
// 
//         for (s, l) in module_levels.iter() {
//             simple_logger = simple_logger.with_module_level(s, *l);
//         }
// 
//         simple_logger.with_utc_timestamps().init().unwrap();
//     }
// }

/// Initialize env logger.
pub fn initialize_env_logger(module_levels: &Vec<(String, LevelFilter)>) {

    #[cfg(not(target_arch = "wasm32"))]
    {

        let mut env_logger = env_logger::builder();

        for (s, l) in module_levels.iter() {
            env_logger.filter_module(s, *l);
        }
        env_logger
            .filter_level(log::LevelFilter::Info)
            //.filter_module("wgpu_core", log::LevelFilter::Info)
            // .filter_module("wgpu_hal", log::LevelFilter::Info)
            // .filter_module("naga", log::LevelFilter::Info)
            // .filter_module("wgpu_hal", log::LevelFilter::Error)
            // .filter_module("naga", log::LevelFilter::Error)
            .parse_default_env().init();
    }
}
