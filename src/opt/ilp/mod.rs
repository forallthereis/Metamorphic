use std::cmp::Ordering;

pub mod solve;





type VarValueType = i16;



struct ILPVariable {
    pub name: String,
    pub lower_bound: VarValueType,
    pub upper_bound: VarValueType,
}

impl ILPVariable {

    #[inline]
    fn range(&self) -> usize {
        (self.upper_bound - self.lower_bound) as usize + 1
    }

}



struct ILPConstraint {
    pub name: String,
    pub rel: Ordering,
    pub rhs: f64,
}
