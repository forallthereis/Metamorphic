use std::sync::{Arc, Mutex};

use chrono::Utc;

use crate::{io::log::Loggable, opt::Solver};

pub mod incidence_graph;





pub(crate) fn solve_with_incidence_graph(
    config: &crate::Config,
    logger: Arc<Mutex<crate::io::log::Logger>>
) {
    logger.lock().unwrap().log_info("Reading the input file...");

    let input_read_start_time = Utc::now();

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

    let input_read_elapsed_time = Utc::now() - input_read_start_time;

    logger.lock().unwrap().log_info(format!(
        "ILP instance '{}' was successfully loaded and its incidence graph was constructed. \
        Vertex count is {}. Edge count is {}. CC count is {}. Solving...",
        instance.name, instance.vertex_count(), instance.edge_count(), instance.cc_count()
    ));

    let solve_start_time = Utc::now();
    instance.solve(Arc::clone(&logger), config.threads_count);
    let solve_elapsed_time = Utc::now() - solve_start_time;

    let obj_value = match instance.solution_obj_value() {
        crate::opt::InstanceStatus::Optimal(value) => {
            logger.lock().unwrap().log_info(format!("The optimal value is {}.", value));
            Some(value)
        },
        crate::opt::InstanceStatus::Feasible(value) => {
            logger.lock().unwrap().log_info(format!("The objective value of the found feasible solution is {}.", value));
            Some(value)
        },
        crate::opt::InstanceStatus::Infeasible => None,
        crate::opt::InstanceStatus::Unsolved => unreachable!(),
    };

    let solution_write_result = match instance.solution() {
        crate::opt::InstanceStatus::Optimal(solution) => crate::io::sol::write(config, &solution, obj_value.unwrap()),
        crate::opt::InstanceStatus::Feasible(solution) => crate::io::sol::write(config, &solution, obj_value.unwrap()),
        crate::opt::InstanceStatus::Infeasible => Ok(()),
        crate::opt::InstanceStatus::Unsolved => unreachable!(),
    };

    match solution_write_result {
        Ok(_) => (),
        Err(e) => {
            logger.lock().unwrap().log_error(e);
            return;
        },
    }

    {
        let mut logger_lock = logger.lock().unwrap();
        logger_lock.log_info(format!("The input file was read in {} s.", input_read_elapsed_time.as_seconds_f64()));
        logger_lock.log_info(format!("The instance was solved in {} s.", solve_elapsed_time.as_seconds_f64()));
        logger_lock.log_info(format!("Total elapsed time is {} s.", input_read_elapsed_time.as_seconds_f64() + solve_elapsed_time.as_seconds_f64()));
    }
}
