use std::cmp::Ordering;

pub mod solve;





type VarValueType = i16;



pub(crate) struct ILPInstance {
    name: String,
    vars: Vec<ILPVariable>,
    opt_goal: super::OptGoal,
    obj_coeffs: Vec<f64>,
    constrs: Vec<ILPConstraint>,
}

impl ILPInstance {

    pub fn add_var(
        &mut self,
        name: String,
        lower_bound: VarValueType,
        upper_bound: VarValueType,
        obj_coeff: f64
    ) -> crate::error::Result<()> {

        if !self.constrs.is_empty() {
            return Err(
                crate::error::MetamorhicError::ForbiddenCommand(format!(
                    "there was an attempt to add variable '{}' to the ILP instance '{}' that already contains constraints. \
                    All variables must be added to the instance before the first constraint is defined.",
                    name, self.name,
                ))
            );
        }

        if self.vars.try_reserve(1).is_err() || self.obj_coeffs.try_reserve(1).is_err() {
            return Err(
                crate::error::MetamorhicError::OutOfMemory(format!(
                    "there was not enough memory to add variable '{}' to the ILP instance '{}'. \
                    Without '{}', the instance contains {} variables.",
                    name, self.name, name, self.vars.len(),
                ))
            );
        }

        self.vars.push(
            ILPVariable { name, lower_bound, upper_bound }
        );
        self.obj_coeffs.push(obj_coeff);

        Ok(())

    }

}



struct ILPVariable {
    name: String,
    lower_bound: VarValueType,
    upper_bound: VarValueType,
}



struct ILPConstraint {
    name: String,
    lhs_coeffs: Vec<f64>,
    rel: Ordering,
    rhs: f64,
}
