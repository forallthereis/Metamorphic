use std::{env, path::PathBuf, sync::{Arc, Mutex}};

mod error;
mod io;
mod opt;





const F64_EQ_ULPS: i64 = 2;
const SUPPORTED_INPUT_FILE_EXTENSIONS: [&str; 2] = ["lp", "mps"];
const VERSION: &str = "0.1.0 (preview)";



fn main() {
    let args: Vec<_> = env::args().collect();

    let config = if args.len() == 1 {
        crate::io::cli::inquire_config()
    } else {
        todo!();
    };

    config.run();
}



struct Config {
    pub input_file_path: PathBuf,
    pub threads_count: usize,
    pub log_format: crate::io::log::LogFormat,
    pub log_destination: crate::io::log::LogDestination,
}

impl Config {
    fn run(&self) {
        let logger = Arc::new(Mutex::new(crate::io::log::Logger::new(self)));

        logger.lock().unwrap().log_info(format!("Metamorphic v.{}\nLog started.", VERSION));

        crate::opt::ilp::solve::solve_with_incidence_graph(self, Arc::clone(&logger));

        logger.lock().unwrap().log_info("Log finished.");
    }
}
