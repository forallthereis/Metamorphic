pub(in crate::opt::ilp) struct ILPIncidenceGraph<'a> {
    instance: &'a crate::opt::ilp::ILPInstance,
    graph: petgraph::Graph<ILPIncidenceGraphVertex, f64, petgraph::Undirected, usize>,
}



enum ILPIncidenceGraphVertex {
    VarVertex(usize),
    ConstrVertex(usize),
}
