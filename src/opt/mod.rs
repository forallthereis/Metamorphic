use std::sync::{Arc, Mutex, mpsc::Sender};

pub mod ilp;





pub(crate) enum OptGoal {
    Min,
    Max,
}



pub(crate) enum InstanceStatus<SolutionType> {
    Unsolved,
    Infeasible,
    Feasible(SolutionType),
    Optimal(SolutionType),
}



pub(crate) trait Solver {
    fn solve(
        &mut self,
        logger: Arc<Mutex<crate::io::log::Logger>>,
        threads_count: usize,
        timeout: f64,
    ) -> crate::error::Result<()>;
}
