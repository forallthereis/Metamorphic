use std::{cmp::Ordering, sync::{Arc, Mutex}};

use dendroverse::{DendroverseInstance, NiceDTDMemo};
use float_cmp::approx_eq;
use fxhash::{FxBuildHasher, FxHashMap};
use petgraph::{Graph, Undirected, graph::NodeIndex};
use polars::prelude::*;





pub(crate) struct IncidenceGraph {
    name: String,
    status: crate::opt::InstanceStatus<FxHashMap<NodeIndex, crate::opt::ilp::VarValueType>>,
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
            if !approx_eq!(f64, coeff, 0.0, ulps = 2) && self.incidence_graph.try_update_edge(eid, self.var_name_map[var_name], coeff).is_err() {
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
        self.obj_coeffs.insert(vid, obj_coeff);

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

    fn solve(
        &mut self,
        logger: Arc<Mutex<crate::io::log::Logger>>,
        threads_count: usize,
        timeout: f64,
    ) -> crate::error::Result<()>
    {
        todo!();
    }

}



enum IncidenceGraphVertex {
    VarVertex(crate::opt::ilp::ILPVariable),
    ConstrVertex(crate::opt::ilp::ILPConstraint),
}



const OBJ_VALUE_COLUMN_NAME: &str = "obj value";



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

        let mut vars = FxHashMap::with_capacity_and_hasher(bag.len(), FxBuildHasher::new());
        let mut constrs = FxHashMap::with_capacity_and_hasher(bag.len(), FxBuildHasher::new());
        for vid in bag {
            let index = (*vid as u32).into();
            match additional_data.incidence_graph.node_weight(index).unwrap() {
                IncidenceGraphVertex::VarVertex(var_info) => {vars.insert(index, var_info);},
                IncidenceGraphVertex::ConstrVertex(constr_info) => {constrs.insert(index, constr_info);},
            }
        }

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

            let new_obj_column =
                &self.records[OBJ_VALUE_COLUMN_NAME]
                +
                &(&self.records[var_column_name.as_str()] * additional_data.obj_coeffs[&var_vid]);
            let new_obj_column = new_obj_column.unwrap();

            self.records.replace(OBJ_VALUE_COLUMN_NAME, new_obj_column).unwrap();

        }

        for (constr_vid, _) in constrs.iter() {

            let constr_column_name: PlSmallStr = format!("c{}", constr_vid.index()).into();
            let constr_column = Column::new(constr_column_name.clone(), vec![0.0; records_count]);

            self.records.with_column(constr_column).unwrap();

            for (var_vid, _) in vars.iter() {

                if let Some(edge) = additional_data.incidence_graph.edges_connecting(*constr_vid, *var_vid).next() {

                    let var_column_name = format!("v{}", var_vid.index());
                    let coeff = *edge.weight();

                    let new_constr_column =
                        &self.records[constr_column_name.as_str()]
                        +
                        &(&self.records[var_column_name.as_str()] * coeff);
                    let new_constr_column = new_constr_column.unwrap();

                    self.records.replace(constr_column_name.as_str(), new_constr_column).unwrap();

                }

            }

        }

    }

    fn forget_introduce_payload(
        &mut self,
        child_bag: &Vec<usize>,
        child_nid: usize,
        child_memo: &Self,
        forgotten_vids: &Vec<usize>,
        introduced_vids: &Vec<usize>,
        additional_data: &IncidenceGraph,
    )
    {
        todo!();
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
        todo!();
    }

}



struct IncidenceGraphMemoRecord {
    var_values: FxHashMap<NodeIndex, crate::opt::ilp::VarValueType>,
    constr_values: FxHashMap<NodeIndex, f64>,
}
