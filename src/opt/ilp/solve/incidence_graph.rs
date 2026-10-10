use std::{cmp::Ordering, sync::{Arc, Mutex}};

use dendroverse::{BacktrackableMemo, DendroverseInstance, NiceDTDMemo};
use float_cmp::approx_eq;
use fxhash::{FxBuildHasher, FxHashMap};
use petgraph::{Graph, Undirected, graph::NodeIndex};
use polars::prelude::*;





// Internally, the optimisation goal is always to maximise
pub(crate) struct IncidenceGraph {
    name: String,
    status: crate::opt::InstanceStatus<IncidenceGraphSolution>,
    obj_coeffs: FxHashMap<NodeIndex, f64>,
    opt_goal: crate::opt::OptGoal,
    incidence_graph: Graph<IncidenceGraphVertex, f64, Undirected, u32>,
    var_name_map: FxHashMap<String, NodeIndex>,
    constr_name_map: FxHashMap<String, NodeIndex>,
}

impl IncidenceGraph {

    pub(crate) fn add_constr(
        &mut self,
        name: String,
        lhs: Vec<(f64, &str)>,
        rel: Ordering,
        rhs: f64,
    ) -> crate::error::Result<()> {

        if self.constr_name_map.try_reserve(1).is_err() {
            crate::return_error!(
                OutOfMemory,
                "there was not enough memory to add constraint '{}' to ILP instance '{}'. \
                Without '{}', the instance contained {} variables and {} constraints at the moment of failure.",
                name, self.name, name, self.var_name_map.len(), self.constr_name_map.len()
            );
        }

        let eid_or_err = self.incidence_graph.try_add_node(
            IncidenceGraphVertex::ConstrVertex(
                crate::opt::ilp::ILPConstraint { name: name.clone(), rel, rhs }
            )
        );

        if eid_or_err.is_err() {
            crate::return_error!(
                OutOfMemory,
                "there was not enough memory to add constraint '{}' to ILP instance '{}'. \
                Without '{}', the instance contained {} variables and {} constraints at the moment of failure.",
                name, self.name, name, self.var_name_map.len(), self.constr_name_map.len()
            );
        }

        let eid = eid_or_err.unwrap();

        for (coeff, var_name) in lhs {
            if !approx_eq!(f64, coeff, 0.0, ulps = crate::F64_EQ_ULPS) && self.incidence_graph.try_update_edge(eid, self.var_name_map[var_name], coeff).is_err() {
                crate::return_error!(
                    OutOfMemory,
                    "there was not enough memory to add constraint '{}' to ILP instance '{}'. \
                    Without '{}', the instance contained {} variables and {} constraints at the moment of failure.",
                    name, self.name, name, self.var_name_map.len(), self.constr_name_map.len()
                );
            }
        }

        Ok(())

    }

    pub(crate) fn add_var(
        &mut self,
        name: String,
        lower_bound: crate::opt::ilp::VarValueType,
        upper_bound: crate::opt::ilp::VarValueType,
        obj_coeff: f64,
    ) -> crate::error::Result<()> {

        if self.var_name_map.try_reserve(1).is_err() || self.obj_coeffs.try_reserve(1).is_err() {
            crate::return_error!(
                OutOfMemory,
                "there was not enough memory to add variable '{}' to ILP instance '{}'. \
                Without '{}', the instance contained {} variables and {} constraints at the moment of failure.",
                name, self.name, name, self.var_name_map.len(), self.constr_name_map.len()
            );
        }

        let vid_or_err = self.incidence_graph.try_add_node(
            IncidenceGraphVertex::VarVertex(
                crate::opt::ilp::ILPVariable { name: name.clone(), lower_bound, upper_bound }
            )
        );

        if vid_or_err.is_err() {
            crate::return_error!(
                OutOfMemory,
                "there was not enough memory to add variable '{}' to ILP instance '{}'. \
                Without '{}', the instance contained {} variables and {} constraints at the moment of failure.",
                name, self.name, name, self.var_name_map.len(), self.constr_name_map.len()
            );
        }

        let vid = vid_or_err.unwrap();
        self.var_name_map.insert(name, vid);
        self.obj_coeffs.insert(
            vid,
            match self.opt_goal {
                crate::opt::OptGoal::Max => obj_coeff,
                crate::opt::OptGoal::Min => -obj_coeff,
            },
        );

        Ok(())

    }

    pub(crate) fn new(name: String, opt_goal: crate::opt::OptGoal) -> Self {

        Self {
            name,
            status: crate::opt::InstanceStatus::Unsolved,
            obj_coeffs: FxHashMap::with_hasher(FxBuildHasher::new()),
            opt_goal,
            incidence_graph: Graph::new_undirected(),
            var_name_map: FxHashMap::with_hasher(FxBuildHasher::new()),
            constr_name_map: FxHashMap::with_hasher(FxBuildHasher::new()),
        }

    }

}

impl crate::opt::Solver for IncidenceGraph {

    fn solution(&self) -> crate::opt::InstanceStatus<FxHashMap<String, f64>> {

        match &self.status {

            crate::opt::InstanceStatus::Optimal(solution) =>
                crate::opt::InstanceStatus::Optimal(
                    FxHashMap::from_iter(
                        solution
                            .vars_map
                            .iter()
                            .map(
                                |(vid, value)|
                                match self.incidence_graph.node_weight(*vid).unwrap() {
                                    IncidenceGraphVertex::VarVertex(var_info) => (var_info.name.clone(), *value),
                                    IncidenceGraphVertex::ConstrVertex(_) => unreachable!(),
                                })
                    )
                ),

            crate::opt::InstanceStatus::Feasible(_) => unreachable!(),

            crate::opt::InstanceStatus::Infeasible => crate::opt::InstanceStatus::Infeasible,

            crate::opt::InstanceStatus::Unsolved => crate::opt::InstanceStatus::Unsolved,

        }

    }

    fn solution_obj_value(&self) -> crate::opt::InstanceStatus<f64> {
        match &self.status {
            crate::opt::InstanceStatus::Optimal(solution) => crate::opt::InstanceStatus::Optimal(solution.opt_value),
            crate::opt::InstanceStatus::Feasible(_) => unreachable!(),
            crate::opt::InstanceStatus::Infeasible => crate::opt::InstanceStatus::Infeasible,
            crate::opt::InstanceStatus::Unsolved => crate::opt::InstanceStatus::Unsolved,
        }
    }

    fn solve(
        &mut self,
        logger: Arc<Mutex<crate::io::log::Logger>>,
        threads_count: usize,
    )
    {

        logger.lock().unwrap().log_info(format!(
            "Computing a nice tree decomposition for ILP instance '{}'...",
            self.name
        ));

        let mut dv_instance: DendroverseInstance<IncidenceGraphMemo, _> =
            match DendroverseInstance::with_auto_generated_nice_dtd(&self.incidence_graph, self) {
                Ok(value) => value,
                Err(_) => {
                    logger.lock().unwrap().log_error(
                        crate::error::MetamorhicError::Generic(format!(
                            "Tree decomposition construction failed for ILP instance '{}'.\
                            Most probable cause is that the incidence graph of the instance is too big to compute its exact tree decomposition.",
                            self.name
                        ))
                    );
                    return;
                },
            };

        logger.lock().unwrap().log_info(
            "A nice tree decomposition was successfully computed. Proceeding with dynamic programming..."
        );

        if dv_instance.solve_using_nice_dtd(threads_count).is_err() {
            logger.lock().unwrap().log_error(
                crate::error::MetamorhicError::Generic(format!(
                    "An error occured while solving instance '{}'.",
                    self.name
                ))
            );
            return;
        };

        logger.lock().unwrap().log_info(format!(
            "ILP instance '{}' was solved. Retrieving an optimal solution...",
            self.name
        ));

        self.status = match dv_instance.solution() {
            Some(solution) => crate::opt::InstanceStatus::Optimal(solution),
            None => crate::opt::InstanceStatus::Infeasible,
        };

        logger.lock().unwrap().log_info(
            match &self.status {
                crate::opt::InstanceStatus::Optimal(_) => format!(
                    "ILP instance '{}' is feasible and an optimal solution was retrieved.",
                    self.name
                ),
                crate::opt::InstanceStatus::Infeasible => format!(
                    "ILP instance '{}' is infeasible, no solution exists.",
                    self.name
                ),
                _ => unreachable!(),
            }
        );

    }

}



enum IncidenceGraphVertex {
    VarVertex(crate::opt::ilp::ILPVariable),
    ConstrVertex(crate::opt::ilp::ILPConstraint),
}



const OBJ_VALUE_COLUMN_NAME: &str = "obj value";
const ROW_INDEX_COLUMN_NAME: &str = "child row index";



#[derive(Default)]
struct IncidenceGraphMemo {
    records: DataFrame,
}

impl NiceDTDMemo<IncidenceGraph> for IncidenceGraphMemo {

    fn leaf_payload(
        &mut self,
        bag: &Vec<usize>,
        additional_data: &IncidenceGraph,
    )
    {

        let (vars, constrs) = split_vec_into_vars_and_constrs(bag, additional_data);

        let mut repeat_var_value_times: usize = vars
            .iter()
            .map(|(_, var_info)| var_info.range())
            .product();
        let records_count = repeat_var_value_times;

        let obj_column = Column::new(OBJ_VALUE_COLUMN_NAME.into(), vec![0.0; records_count]);
        self.records.with_column(obj_column).unwrap();

        for (var_vid, var_info) in vars.iter() {

            repeat_var_value_times /= var_info.range();

            let var_column_name: PlSmallStr = format!("v{}", var_vid.index()).into();
            let var_column_vec = Vec::from_iter(
                (0..records_count)
                    .map(|record_i| (var_info.lower_bound + (record_i / repeat_var_value_times % var_info.range()) as i16) as f64)
            );
            let var_column = Column::new(var_column_name.clone(), var_column_vec);

            self.records.with_column(var_column).unwrap();

            self.records = self.records
                .clone()
                .lazy()
                .with_column(
                    (
                        col(OBJ_VALUE_COLUMN_NAME) + lit(additional_data.obj_coeffs[&var_vid]) * col(var_column_name)
                    ).alias(OBJ_VALUE_COLUMN_NAME)
                )
                .collect()
                .unwrap();

        }

        for (constr_vid, _) in constrs.iter() {

            let constr_column_name: PlSmallStr = format!("c{}", constr_vid.index()).into();
            let constr_column = Column::new(constr_column_name.clone(), vec![0.0; records_count]);

            self.records.with_column(constr_column).unwrap();

            for (var_vid, _) in vars.iter() {

                if let Some(edge) = additional_data.incidence_graph.edges_connecting(*constr_vid, *var_vid).next() {

                    let var_column_name = format!("v{}", var_vid.index());
                    let coeff = *edge.weight();

                    self.records = self.records
                        .clone()
                        .lazy()
                        .with_column(
                            (
                                col(constr_column_name.clone()) + lit(coeff) * col(var_column_name)
                            ).alias(constr_column_name.clone())
                        )
                        .collect()
                        .unwrap();

                }

            }

        }

    }

    fn forget_introduce_payload(
        &mut self,
        child_bag: &Vec<usize>,
        _child_nid: usize,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        introduced_vids: &Vec<usize>,
        additional_data: &IncidenceGraph,
    )
    {

        let (vars, constrs) = split_vec_into_vars_and_constrs(child_bag, additional_data);
        let (forgotten_vars, forgotten_constrs) = split_vec_into_vars_and_constrs(forgotten_vids, additional_data);
        let (introduced_vars, introduced_constrs) = split_vec_into_vars_and_constrs(introduced_vids, additional_data);

        let mut intermediate_records = child_memo.records
            .clone()
            .lazy()
            .select([all().exclude_cols([format!("^{}*$", ROW_INDEX_COLUMN_NAME)]).as_expr()])
            .with_row_index(ROW_INDEX_COLUMN_NAME, None);

        // Forget vertices
        // Filter out the records where at least one forgotten constraint isn't satsfied
        // Then remove all columns corresponding to the forgotten vertices and group the remaining ones while aggregating the objective value
        intermediate_records = intermediate_records
            .filter(
                forgotten_constrs
                    .iter()
                    .map(
                        |(constr_vid, constr_info)| {
                            let constr_column_name = format!("c{}", constr_vid.index());
                            match constr_info.rel {
                                Ordering::Equal => {
                                    let (eq_lower_bound, eq_upper_bound) = ulp_bounds(constr_info.rhs);
                                    col(constr_column_name.clone()).gt_eq(eq_lower_bound).and(col(constr_column_name).lt_eq(eq_upper_bound))
                                },
                                Ordering::Greater => col(constr_column_name).gt_eq(constr_info.rhs),
                                Ordering::Less => col(constr_column_name).lt_eq(constr_info.rhs),
                            }
                        }
                    )
                    .fold(lit(true), |acc, expr| acc.and(expr))
            )
            .drop(cols(forgotten_vars.iter().map(|(var_vid, _)| format!("v{}", var_vid.index()))))
            .drop(cols(forgotten_constrs.iter().map(|(constr_vid, _)| format!("c{}", constr_vid.index()))))
            .group_by([all().exclude_cols([ROW_INDEX_COLUMN_NAME, OBJ_VALUE_COLUMN_NAME])])
            .agg(
                [
                    col(OBJ_VALUE_COLUMN_NAME).max().alias(OBJ_VALUE_COLUMN_NAME),
                    col(ROW_INDEX_COLUMN_NAME).max_by(col(OBJ_VALUE_COLUMN_NAME)).alias(ROW_INDEX_COLUMN_NAME),
                ]
            );

        // Introduce vertices
        for (var_vid, var_info) in introduced_vars.iter() {

            let var_column_name: PlSmallStr = format!("v{}", var_vid.index()).into();
            let var_values = df!(var_column_name.clone() => Vec::from_iter((var_info.lower_bound..=var_info.upper_bound).map(|value| value as f64))).unwrap().lazy();

            intermediate_records = intermediate_records
                .cross_join(var_values, None)
                .with_column(
                    (
                        col(OBJ_VALUE_COLUMN_NAME) + lit(additional_data.obj_coeffs[var_vid]) * col(var_column_name)
                    ).alias(OBJ_VALUE_COLUMN_NAME)
                );

        }

        for (constr_vid, _) in constrs.iter() {

            let constr_column_name: PlSmallStr = format!("c{}", constr_vid.index()).into();

            for (var_vid, _) in introduced_vars.iter() {

                if let Some(edge) = additional_data.incidence_graph.edges_connecting(*constr_vid, *var_vid).next() {

                    let var_column_name = format!("v{}", var_vid.index());
                    let coeff = *edge.weight();

                    intermediate_records = intermediate_records
                        .with_column(
                            (
                                col(constr_column_name.clone()) + lit(coeff) * col(var_column_name)
                            ).alias(constr_column_name.clone())
                        );

                }

            }

        }

        for (constr_vid, _) in introduced_constrs.iter() {

            let constr_column_name: PlSmallStr = format!("c{}", constr_vid.index()).into();

            intermediate_records = intermediate_records
                .with_column(
                    (
                        lit(0.0)
                    ).alias(constr_column_name.clone())
                );

            let new_vars = vars
                .iter()
                .filter(|(var_vid, _)| !forgotten_vars.contains_key(*var_vid))
                .chain(introduced_vars.iter());

            for (var_vid, _) in new_vars {

                if let Some(edge) = additional_data.incidence_graph.edges_connecting(*constr_vid, *var_vid).next() {

                    let var_column_name = format!("v{}", var_vid.index());
                    let coeff = *edge.weight();

                    intermediate_records = intermediate_records
                        .with_column(
                            (
                                col(constr_column_name.clone()) + lit(coeff) * col(var_column_name)
                            ).alias(constr_column_name.clone())
                        );

                }

            }

        }

        self.records = intermediate_records.collect().unwrap();

    }

    fn join_payload(
        &mut self,
        children_bag: &Vec<usize>,
        child1_nid: usize,
        child1_memo: &Self,
        child2_nid: usize,
        child2_memo: &Self,
        additional_data: &IncidenceGraph,
    )
    {

        let (vars, constrs) = split_vec_into_vars_and_constrs(children_bag, additional_data);

        let intermediate_child1_records = child1_memo.records
            .clone()
            .lazy()
            .select([all().exclude_cols([format!("^{}.*$", ROW_INDEX_COLUMN_NAME)]).as_expr()])
            .with_row_index(format!("{}{}", ROW_INDEX_COLUMN_NAME, child1_nid), None);
        let intermediate_child2_records = child2_memo.records
            .clone()
            .lazy()
            .select([all().exclude_cols([format!("^{}*$", ROW_INDEX_COLUMN_NAME)]).as_expr()])
            .with_row_index(format!("{}{}", ROW_INDEX_COLUMN_NAME, child2_nid), None);

        let joined_column_names: Vec<_> = vars
            .iter()
            .map(|(var_vid, _)| col(format!("v{}", var_vid.index())))
            .collect();

        let mut intermediate_records = intermediate_child1_records
            .join(
                intermediate_child2_records,
                joined_column_names.clone(),
                joined_column_names,
                JoinArgs::new(JoinType::Inner),
            );

        for (constr_vid, _) in constrs {

            let constr_column_name: PlSmallStr = format!("c{}", constr_vid.index()).into();
            let constr_column_name2: PlSmallStr = format!("c{}_right", constr_vid.index()).into();

            intermediate_records = intermediate_records
                .with_column(
                    (
                        col(constr_column_name.clone()) + col(constr_column_name2.clone())
                    ).alias(constr_column_name.clone())
                )
                .drop(cols([constr_column_name2]));

            for (var_vid, _) in vars.iter() {

                if let Some(edge) = additional_data.incidence_graph.edges_connecting(constr_vid, *var_vid).next() {

                    let var_column_name = format!("v{}", var_vid.index());
                    let coeff = *edge.weight();

                    intermediate_records = intermediate_records
                        .with_column(
                            (
                                col(constr_column_name.clone()) - lit(coeff) * col(var_column_name)
                            ).alias(constr_column_name.clone())
                        );

                }

            }

        }

        intermediate_records = intermediate_records
            .with_column(
                (
                    col(OBJ_VALUE_COLUMN_NAME) + col(format!("{}_right", OBJ_VALUE_COLUMN_NAME))
                ).alias(OBJ_VALUE_COLUMN_NAME)
            )
            .drop(cols([format!("{}_right", OBJ_VALUE_COLUMN_NAME)]));

        for (var_vid, _) in vars {

            intermediate_records = intermediate_records
                .with_column(
                    (
                        col(OBJ_VALUE_COLUMN_NAME) - lit(additional_data.obj_coeffs[&var_vid]) * col(format!("v{}", var_vid.index()))
                    ).alias(OBJ_VALUE_COLUMN_NAME)
                );

        }

        self.records = intermediate_records.collect().unwrap();

    }

}

impl BacktrackableMemo for IncidenceGraphMemo {

    type BacktrackingHint = usize;
    type PartialSolutionType = IncidenceGraphSolution;
    type SolutionType = IncidenceGraphSolution;

    fn extend_partial_solution(
        &self,
        _nid: usize,
        mut partial_solution: Self::PartialSolutionType,
        hint: Option<Self::BacktrackingHint>,
        children_nids: &Vec<usize>
    ) -> (Option<Self::PartialSolutionType>, Vec<Option<Self::BacktrackingHint>>)
    {

        let mut next_hint = Vec::with_capacity(2);

        let opt_row_idx = match hint {
            Some(value) => value,
            None => match self.records[OBJ_VALUE_COLUMN_NAME].as_materialized_series().arg_max() {
                Some(value) => {
                    partial_solution.opt_value += self.records[OBJ_VALUE_COLUMN_NAME].f64().unwrap().get(value).unwrap();
                    value
                },
                None => return (None, vec![None; children_nids.len()]),
            },
        };

        for column_name in self.records.get_column_names() {
            if column_name.starts_with("v") {
                let var_vid: NodeIndex<u32> = NodeIndex::new(column_name[1..].parse().unwrap());
                partial_solution.vars_map.insert(var_vid, self.records[column_name.as_str()].f64().unwrap().get(opt_row_idx).unwrap());
            }
        }

        match children_nids.len() {
            0 => (),
            1 => next_hint.push(Some(
                self
                    .records[ROW_INDEX_COLUMN_NAME]
                    .u32()
                    .unwrap()
                    .get(opt_row_idx)
                    .unwrap() as usize
            )),
            _ => {
                for child_nid in children_nids {
                    next_hint.push(Some(
                        self
                            .records[format!("{}{}", ROW_INDEX_COLUMN_NAME, child_nid).as_str()]
                            .u32()
                            .unwrap()
                            .get(opt_row_idx)
                            .unwrap() as usize
                    ));
                }
            },
        }

        (Some(partial_solution), next_hint)

    }

}



#[derive(Default)]
struct IncidenceGraphSolution {
    opt_value: f64,
    vars_map: FxHashMap<NodeIndex, f64>,
}



fn split_vec_into_vars_and_constrs<'a>(
    bag: &Vec<usize>,
    incidence_graph: &'a IncidenceGraph,
) -> (FxHashMap<NodeIndex, &'a crate::opt::ilp::ILPVariable>, FxHashMap<NodeIndex, &'a crate::opt::ilp::ILPConstraint>) {

    let mut vars = FxHashMap::with_capacity_and_hasher(bag.len(), FxBuildHasher::new());
    let mut constrs = FxHashMap::with_capacity_and_hasher(bag.len(), FxBuildHasher::new());
    for vid in bag {
        let index = (*vid as u32).into();
        match incidence_graph.incidence_graph.node_weight(index).unwrap() {
            IncidenceGraphVertex::VarVertex(var_info) => {vars.insert(index, var_info);},
            IncidenceGraphVertex::ConstrVertex(constr_info) => {constrs.insert(index, constr_info);},
        }
    }

    (vars, constrs)

}

fn ulp_bounds(x: f64) -> (f64, f64) {

    let mut lower = x;
    let mut upper = x;

    for _ in 0..crate::F64_EQ_ULPS {
        lower = lower.next_down();
        upper = upper.next_up();
    }

    (lower, upper)

}
