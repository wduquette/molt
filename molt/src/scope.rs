//! Variable Scope Stack
//!
//! A scope contains the variables for a given level in the call stack.  New scopes are pushed
//! onto the stack by procedure on entry and popped on exit.  Variables in the current scope
//! can be mapped to variables in lower scopes (e.g., scope #0, the `global` scope) using
//! the `upvar` method.
//!
//! Scopes are numbered starting at `0`, the `global` scope.  Scopes with lower indices than
//! the current are said to be higher in the stack, following Standard TCL practice (e.g.,
//! `upvar`, `uplevel`).
//!
//! Molt clients do not interact with this mechanism directly, but via the
//! `Interp` (or the Molt language itself).

use crate::types::MoltList;
use crate::types::MoltResult;
use crate::value::Value;
use std::collections::HashMap;

/// A variable in a `Scope`.  If the variable is defined in the `Scope`, it has a
/// `Value`; if it is a reference to a variable in a higher scope (e.g., a global) then
/// the `Level` gives the referenced scope.
enum Var {
    Scalar(Value),
    // Array(HashMap<String, Value>),
    Level(usize),
}

/// A scope: a level in the `ScopeStack`.  It contains a hash table of `Var`'s by name.
#[derive(Default)]
struct Scope {
    /// Vars in this scope by name.
    map: HashMap<String, Var>,
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Scope {
            map: HashMap::new(),
        }
    }
}

/// The scope stack: a stack of variable scopes corresponding to the Molt `proc`
/// call stack.
#[derive(Default)]
pub(crate) struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
    /// Creates a scope stack containing only scope `0`, the global scope.
    pub fn new() -> Self {
        let mut ss = Self { stack: Vec::new() };

        ss.stack.push(Scope::new());

        ss
    }

    /// Sets a variable to a value in the current scope.  If the variable is linked to
    /// another scope, the value is set there instead.  The variable is created if it does
    /// not already exist.
    pub fn set(&mut self, name: &str, value: Value) -> MoltResult {
        let top = self.stack.len() - 1;

        self.set_at(top, name, value.clone());
        Ok(value)
    }

    /// Gets the value of the named variable in the current scope, if present.
    pub fn get(&self, name: &str) -> Option<Value> {
        let top = self.stack.len() - 1;

        self.get_at(top, name)
    }

    /// Unsets a variable in the current scope, i.e., removes it from the scope.
    /// If the variable is a reference to another scope, the variable is removed from that
    /// scope as well.
    pub fn unset(&mut self, name: &str) {
        let top = self.stack.len() - 1;
        self.unset_at(top, name);
    }

    /// Gets the value at the given level, recursing up the stack as needed.
    fn get_at(&self, level: usize, name: &str) -> Option<Value> {
        match self.stack[level].map.get(name) {
            Some(Var::Scalar(value)) => Some(value.clone()),
            Some(Var::Level(at)) => self.get_at(*at, name),
            _ => None,
        }
    }

    /// Set a variable to a value at a given level in the stack.  If the variable at that level
    /// is linked to a higher level, sets it at that level instead.
    fn set_at(&mut self, level: usize, name: &str, value: Value) {
        match self.stack[level].map.get(name) {
            Some(Var::Level(at)) => {
                let true_level = *at;
                self.set_at(true_level, name, value);
            }
            _ => {
                self.stack[level].map.insert(name.into(), Var::Scalar(value));
            }
        }
    }

    /// Unset a variable at a given level in the stack.  If the variable at that level
    /// is linked to a higher level, follows the chain down, unsetting as it goes.
    fn unset_at(&mut self, level: usize, name: &str) {
        // FIRST, if the variable at this level links to a lower level, follow the chain.
        if let Some(Var::Level(at)) = self.stack[level].map.get(name) {
            let true_level = *at;
            self.unset_at(true_level, name);
        }

        // NEXT, remove the link at this level.
        self.stack[level].map.remove(name);
    }

    /// Links a variable in the current scope to variable at the given level, counting
    /// from `0`, the global scope.
    ///
    /// **Note:** does not try to create the variable at the referenced scope level, if it
    /// does not exist; the variable will be created on the first `set`, if any.  This is
    /// consistent with standard TCL behavior.
    pub fn upvar(&mut self, level: usize, name: &str) {
        assert!(level < self.current(), "Can't upvar to current stack level");
        let top = self.current();
        self.stack[top].map.insert(name.into(), Var::Level(level));
    }

    /// Returns the index of the current stack level, counting from 0, the global scope.
    /// The current stack level has the highest index, but is said to be the lowest stack
    /// level.
    pub fn current(&self) -> usize {
        self.stack.len() - 1
    }

    /// Pushes a new scope onto the stack.  The scope contains no variables by default, though
    /// the procedure that is pushing it onto the stack will often add some.
    pub fn push(&mut self) {
        self.stack.push(Scope::new());
    }

    /// Pops the current scope from the stack. Panics if we're at the global scope; this implies an
    /// coding error at the Rust level.
    pub fn pop(&mut self) {
        self.stack.pop();
        assert!(!self.stack.is_empty(), "Popped global scope!");
    }

    /// Gets the names of the variables defined in the current scope.
    pub fn vars_in_scope(&self) -> MoltList {
        let top = self.stack.len() - 1;
        self.stack[top]
            .map
            .keys()
            .cloned()
            .map(|x| Value::from(&x))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let ss = ScopeStack::new();
        assert_eq!(ss.stack.len(), 1);
        assert_eq!(ss.current(), 0);
    }

    #[test]
    fn test_set_get_basic() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        let out = ss.get("a");
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "1");

        let _ = ss.set("b", Value::from("2"));
        let out = ss.get("b");
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "2");

        assert!(ss.get("c").is_none());
    }

    #[test]
    fn test_unset_basic() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        assert!(ss.get("a").is_some());
        ss.unset("a");
        assert!(ss.get("a").is_none());
    }

    #[test]
    fn test_push() {
        let mut ss = ScopeStack::new();
        ss.push();
        assert_eq!(ss.stack.len(), 2);
        ss.push();
        assert_eq!(ss.stack.len(), 3);
    }

    #[test]
    fn test_pop() {
        let mut ss = ScopeStack::new();
        ss.push();
        ss.push();
        assert_eq!(ss.stack.len(), 3);
        ss.pop();
        assert_eq!(ss.stack.len(), 2);
        ss.pop();
        assert_eq!(ss.stack.len(), 1);
    }

    #[test]
    #[should_panic]
    fn test_pop_global_scope() {
        let mut ss = ScopeStack::new();
        assert_eq!(ss.stack.len(), 1);
        ss.pop();
    }

    #[test]
    fn test_current() {
        let mut ss = ScopeStack::new();
        assert_eq!(ss.current(), 0);
        ss.push();
        assert_eq!(ss.current(), 1);
        ss.push();
        assert_eq!(ss.current(), 2);
        ss.pop();
        assert_eq!(ss.current(), 1);
        ss.pop();
        assert_eq!(ss.current(), 0);
    }

    #[test]
    fn test_set_levels() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        let _ = ss.set("b", Value::from("2"));

        ss.push();
        assert!(ss.get("a").is_none());
        assert!(ss.get("b").is_none());
        assert!(ss.get("c").is_none());

        let _ = ss.set("a", Value::from("3"));
        let _ = ss.set("b", Value::from("4"));
        let _ = ss.set("c", Value::from("5"));
        assert_eq!(ss.get("a").unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().as_str(), "4");
        assert_eq!(ss.get("c").unwrap().as_str(), "5");

        ss.pop();
        assert_eq!(ss.get("a").unwrap().as_str(), "1");
        assert_eq!(ss.get("b").unwrap().as_str(), "2");
        assert!(ss.get("c").is_none());
    }

    #[test]
    fn test_set_get_upvar() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        let _ = ss.set("b", Value::from("2"));

        ss.push();
        ss.upvar(0, "a");
        assert_eq!(ss.get("a").unwrap().as_str(), "1");
        assert!(ss.get("b").is_none());

        let _ = ss.set("a", Value::from("3"));
        let _ = ss.set("b", Value::from("4"));
        assert_eq!(ss.get("a").unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().as_str(), "4");

        ss.pop();
        assert_eq!(ss.get("a").unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().as_str(), "2");
    }

    #[test]
    fn test_unset_levels() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        let _ = ss.set("b", Value::from("2"));

        ss.push();
        let _ = ss.set("a", Value::from("3"));

        ss.unset("a"); // Was set in this scope
        ss.unset("b"); // Was not set in this scope

        ss.pop();
        assert_eq!(ss.get("a").unwrap().as_str(), "1");
        assert_eq!(ss.get("b").unwrap().as_str(), "2");
    }

    #[test]
    fn test_unset_upvar() {
        let mut ss = ScopeStack::new();

        // Set a value at level 0
        let _ = ss.set("a", Value::from("1"));
        assert!(ss.get("a").is_some());
        ss.push();
        assert!(ss.get("a").is_none());

        // Link a@1 to a@0
        ss.upvar(0, "a");
        assert!(ss.get("a").is_some());

        // Unset it; it should be unset in both scopes.
        ss.unset("a");

        assert!(ss.get("a").is_none());
        ss.pop();
        assert!(ss.get("a").is_none());
    }

    #[test]
    fn test_vars_in_scope() {
        let mut ss = ScopeStack::new();
        // No vars initially
        assert_eq!(ss.vars_in_scope().len(), 0);

        // Add two vars to current scope
        let _ = ss.set("a", Value::from("1"));
        let _ = ss.set("b", Value::from("2"));
        assert_eq!(ss.vars_in_scope().len(), 2);
        assert!(ss.vars_in_scope().contains(&Value::from("a")));
        assert!(ss.vars_in_scope().contains(&Value::from("b")));

        // Push a scope; no vars initially
        ss.push();
        assert_eq!(ss.vars_in_scope().len(), 0);

        // Add a var
        let _ = ss.set("c", Value::from("3"));
        assert_eq!(ss.vars_in_scope().len(), 1);
        assert!(ss.vars_in_scope().contains(&Value::from("c")));

        // Upvar a var
        ss.upvar(0, "a");
        assert_eq!(ss.vars_in_scope().len(), 2);
        assert!(ss.vars_in_scope().contains(&Value::from("a")));

        // Pop a scope
        ss.pop();
        assert_eq!(ss.vars_in_scope().len(), 2);
        assert!(!ss.vars_in_scope().contains(&Value::from("c")));

        // Unset a var
        ss.unset("b");
        assert_eq!(ss.vars_in_scope().len(), 1);
        assert!(!ss.vars_in_scope().contains(&Value::from("b")));
    }
}
