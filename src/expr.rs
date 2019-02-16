//! The Expr command and parser

use crate::*;
use crate::interp::Interp;

/// # expr expr
///
/// Evaluates an expression and returns its result.
pub fn cmd_expr(_interp: &mut Interp, argv: &[&str]) -> InterpResult {
    check_args(1, argv, 2, 2, "expr")?;

    molt_err!("TODO")
}
