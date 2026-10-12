use std::sync::{Arc, Mutex};

use chrono::Local;

use crate::{io::log::Loggable, opt::Solver};

pub mod incidence_graph;





pub(crate) fn solve_with_incidence_graph(
    config: &crate::Config,
    logger: Arc<Mutex<crate::io::log::Logger>>
) {

    logger.lock().unwrap().log_info("Reading the input file...");

    let input_read_start_time = Local::now();

    let reader: Box<dyn crate::io::Reader> = match config.input_file_path.extension().unwrap().to_ascii_lowercase().to_str().unwrap() {
        "lp" => Box::new(crate::io::lp::LPReader),
        "mps" => Box::new(crate::io::mps::MPSReader),
        _ => unreachable!(),
    };

    let raw_instance = match reader.read(&config.input_file_path) {
        Ok(value) => value,
        Err(e) => {
            logger.lock().unwrap().log_error(e.log_content());
            return;
        },
    };

    let mut instance = match incidence_graph::IncidenceGraph::try_from(raw_instance) {
        Ok(value) => value,
        Err(e) => {
            logger.lock().unwrap().log_error(e.log_content());
            return;
        },
    };

    let input_read_elapsed_time = Local::now() - input_read_start_time;

    logger.lock().unwrap().log_info(format!("ILP instance '{}' was successfully loaded. Solving...", instance.name));

    let solve_start_time = Local::now();

    instance.solve(Arc::clone(&logger), config.threads_count);

    let solve_elapsed_time = Local::now() - solve_start_time;

    match instance.solution_obj_value() {
        crate::opt::InstanceStatus::Optimal(value) => logger.lock().unwrap().log_info(format!("The optimal value is {}.", value)),
        crate::opt::InstanceStatus::Feasible(value) => logger.lock().unwrap().log_info(format!("The objective value of the found feasible solution is {}.", value)),
        crate::opt::InstanceStatus::Infeasible => (),
        crate::opt::InstanceStatus::Unsolved => unreachable!(),
    }

    {
        let mut logger_lock = logger.lock().unwrap();
        logger_lock.log_info(format!("The input file was read in {} s.", input_read_elapsed_time.as_seconds_f64()));
        logger_lock.log_info(format!("The instance was solved in {} s.", solve_elapsed_time.as_seconds_f64()));
    }

}
