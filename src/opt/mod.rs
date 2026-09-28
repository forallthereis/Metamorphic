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
