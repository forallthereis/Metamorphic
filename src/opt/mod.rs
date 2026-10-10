use std::{sync::{Arc, Mutex}};

use fxhash::FxHashMap;

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

    fn solution(&self) -> InstanceStatus<FxHashMap<String, f64>>;

    fn solution_obj_value(&self) -> InstanceStatus<f64>;

    fn solve(
        &mut self,
        logger: Arc<Mutex<crate::io::log::Logger>>,
        threads_count: usize,
    );

}
