use std::cmp::Ordering;

pub mod solve;





type VarValueType = i16;



struct ILPVariable {
    name: String,
    lower_bound: VarValueType,
    upper_bound: VarValueType,
}



struct ILPConstraint {
    name: String,
    rel: Ordering,
    rhs: f64,
}
