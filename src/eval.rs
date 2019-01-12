use crate::{Interp, InterpResult};

// This is a copy of eval to be written following the Tcl 7.6 model
pub fn eval(interp: &mut Interp, script: &str) -> InterpResult {
   Ok("".into())
}
