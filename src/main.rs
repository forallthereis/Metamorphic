use std::{env, path::PathBuf, sync::{Arc, Mutex}};

use crate::opt::Solver;

mod error;
mod io;
mod opt;





const F64_EQ_ULPS: i64 = 2;
const SUPPORTED_INPUT_FILE_EXTENSIONS: [&str; 1] = ["lp"];



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
    pub log_format: crate::io::log::LogFormat,
    pub log_destination: crate::io::log::LogDestination,
}

impl Config {

    fn run(&self) {

        let logger = Arc::new(Mutex::new(crate::io::log::Logger::new(self)));

        logger.lock().unwrap().log_info("Metamorphic v.0.1.0\nLog started.");
        logger.lock().unwrap().log_info("Reading the given input file...");

        let mut test_instance = crate::opt::ilp::solve::incidence_graph::IncidenceGraph::new("Test".to_string(), crate::opt::OptGoal::Max);

        test_instance.add_var("x1".to_string(), 0, 1, 2.0).unwrap();
        test_instance.add_var("x2".to_string(), 0, 1, 2.0).unwrap();
        test_instance.add_var("x3".to_string(), 0, 1, 1.0).unwrap();

        test_instance.add_constr("c1".to_string(), vec![(1.0, "x1"), (-1.0, "x2")], std::cmp::Ordering::Less, 0.0).unwrap();
        test_instance.add_constr("c2".to_string(), vec![(1.0, "x2"), (1.0, "x3")], std::cmp::Ordering::Less, 1.0).unwrap();

        test_instance.solve(Arc::clone(&logger), 1);

        logger.lock().unwrap().log_info("Log finished.");

    }

}
