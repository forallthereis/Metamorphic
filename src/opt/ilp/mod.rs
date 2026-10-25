use std::cmp::Ordering;

pub mod solve;





type VarValueType = i16;



struct Variable {
    pub name: String,
    pub lower_bound: VarValueType,
    pub upper_bound: VarValueType,
}

impl Variable {
    #[inline]
    fn range(&self) -> usize {
        (self.upper_bound as usize - self.lower_bound as usize) + 1
    }
}



struct Constraint {
    pub rel: Ordering,
    pub rhs: f64,
}
