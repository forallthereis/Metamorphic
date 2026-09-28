use std::cmp::Ordering;

use float_cmp::approx_eq;
use fxhash::{FxBuildHasher, FxHashMap};
use petgraph::{Graph, Undirected, graph::NodeIndex};





pub(crate) struct IncidenceGraphSolver {
    name: String,
    obj_coeffs: Vec<f64>,
    opt_goal: crate::opt::OptGoal,
    incidence_graph: Graph<ILPIncidenceGraphVertex, f64, Undirected, u32>,
    var_name_map: FxHashMap<String, NodeIndex>,
    constr_name_map: FxHashMap<String, NodeIndex>,
}

impl IncidenceGraphSolver {

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
            ILPIncidenceGraphVertex::ConstrVertex(
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
            ILPIncidenceGraphVertex::VarVertex(
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
        self.obj_coeffs.push(obj_coeff);

        Ok(())

    }

    pub(crate) fn new(name: String, opt_goal: crate::opt::OptGoal) -> Self {

        Self {
            name,
            obj_coeffs: vec![],
            opt_goal,
            incidence_graph: Graph::new_undirected(),
            var_name_map: FxHashMap::with_hasher(FxBuildHasher::new()),
            constr_name_map: FxHashMap::with_hasher(FxBuildHasher::new()),
        }

    }

}



enum ILPIncidenceGraphVertex {
    VarVertex(crate::opt::ilp::ILPVariable),
    ConstrVertex(crate::opt::ilp::ILPConstraint),
}
