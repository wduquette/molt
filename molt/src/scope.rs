//! Variable Scope Stack
//!
//! A scope contains the variables for a given level in the call stack.  New scopes are pushed
//! onto the stack by procedure on entry and popped on exit.  Variables in the current scope
//! can be mapped to variables in higher scopes (e.g., scope 0, the `global` scope) using
//! the `upvar` method.
//!
//! Scopes are numbered starting at `0`, the `global` scope.  Scopes with lower indices than
//! the current are said to be higher in the stack, following Standard TCL practice (e.g.,
//! `upvar`, `uplevel`).
//!
//! Molt clients do not interact with this mechanism directly, but via the
//! `Interp` (or the Molt language itself).

use crate::types::MoltList;
use crate::types::ResultCode;
use crate::value::Value;
use std::collections::HashMap;
use std::fmt::Debug;

/// A variable in a `Scope`.  If the variable is defined in the given `Scope`, it is a
/// `Scalar` or an `Array`; if it is an alias to a variable in a higher scope (e.g., a global)
/// then the `Upvar` gives the referenced scope.  The `New` variant is used transiently as
/// part of setting a variable for the first time.
#[derive(Eq, PartialEq)]
enum Var {
    /// A scalar variable, with its value.
    Scalar(Value),

    /// An array variable, with its hash table from names to values.
    Array(HashMap<String, Value>),

    /// An alias to a variable at a higher stack level, with the referenced stack level.
    /// Note that aliases can chain.
    Upvar(usize),

    /// A variable that has just been created so that it can be set.
    New,
}

impl Debug for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Var::Scalar(value) => write!(f, "Var::Scalar({})", value.as_str()),
            Var::Array(_) => write!(f, "Var::Array(TODO)"),
            Var::Upvar(level) => write!(f, "Var::Upvar({})", level),
            Var::New => write!(f, "Var::New"),
        }
    }
}

/// A scope: a level in the `ScopeStack`.  It contains a hash table of `Var`'s by name.
/// Scopes may be pushed onto the stack and popped off later.  Most typically, a scope is
/// pushed on the stack by a `proc` before executing its body, and then popped afterwards.
#[derive(Default, Debug)]
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
#[derive(Default, Debug)]
pub(crate) struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
    //-------------------------------------------------------------------------------------
    // Public API

    /// Creates a scope stack containing only scope `0`, the global scope.  This is usually
    /// done once, as part of creating an `Interp`.
    pub fn new() -> Self {
        let mut ss = Self { stack: Vec::new() };

        ss.stack.push(Scope::new());

        ss
    }

    /// Gets the value of the named scalar variable in the current scope, if present.
    ///
    /// There are three possible outcomes:
    ///
    /// * The variable exists and is a scalar; `Ok(Some(_))` is returned.
    /// * The variable does not exist; `Ok(None)` is returned.
    /// * The variable exists and is an array: `Err(_)` is returned.
    pub fn get(&self, name: &str) -> Result<Option<Value>, ResultCode> {
        match self.var(self.current(), name) {
            Some(Var::Scalar(value)) => Ok(Some(value.clone())),
            Some(Var::Array(_)) => molt_err!("can't read \"{}\": variable is array", name),
            Some(_) => unreachable!(),
            _ => Ok(None),
        }
    }

    /// Gets the value of an array element given its variable name and index, if present.
    /// It's an error if the variable exists and isn't an array variable.
    /// Gets the value of the named array element in the current scope, if present.
    ///
    /// There are three possible outcomes:
    ///
    /// * The variable and the indexed element both exist; `Ok(Some(_))` is returned.
    /// * Either the variable or the indexed element does not exist; `Ok(None)` is returned.
    /// * The variable exists and is a scalar: `Err(_)` is returned.
    pub fn get_elem(&self, name: &str, index: &str) -> Result<Option<Value>, ResultCode> {
        match self.var(self.current(), name) {
            Some(Var::Scalar(_)) => {
                molt_err!("can't read \"{}({})\": variable isn't array", name, index)
            }
            Some(Var::Array(map)) => {
                if let Some(val) = map.get(index) {
                    Ok(Some(val.clone()))
                } else {
                    Ok(None)
                }
            }
            Some(_) => unreachable!(),
            _ => Ok(None),
        }
    }

    /// Sets the value of the named scalar in the current scope, creating the variable
    /// if it doesn't already exist.  It's an error if the variable exists but is an array
    /// variable.
    pub fn set(&mut self, name: &str, val: Value) -> Result<(), ResultCode> {
        match self.var_mut(self.current(), name) {
            Some(Var::Upvar(_)) => unreachable!(),
            Some(Var::Array(_)) => molt_err!("can't set \"{}\": variable is array", name),
            Some(var) => {
                // It was either Var::Scalar or Var::New; either way, replace it with a new
                // Var::Scalar.
                *var = Var::Scalar(val);
                Ok(())
            }
            None => unreachable!(),
        }
    }

    /// Sets the value of the indexed array element in the current scope, creating the
    /// and/or the element if they don't already exist. It's an error if the variable exists
    /// but is a scalar variable.
    pub fn set_elem(&mut self, name: &str, index: &str, val: Value) -> Result<(), ResultCode> {
        let top = self.current();

        match self.var_mut(top, name) {
            Some(Var::Upvar(_)) => unreachable!(),
            Some(Var::Scalar(_)) => {
                molt_err!("can't set \"{}({})\": variable isn't array", name, index)
            }
            Some(Var::Array(map)) => {
                // It was already an array; just update the indexed element (which will
                // create it if it didn't exist).
                map.insert(index.into(), val);
                Ok(())
            }
            Some(var) => {
                assert_eq!(*var, Var::New);
                // Create new variable on the top of the stack.
                let mut map = HashMap::new();
                map.insert(index.into(), val);
                *var = Var::Array(map);
                Ok(())
            }
            None => unreachable!(),
        }
    }

    /// Unsets a variable in the current scope, i.e., removes it from the scope.
    /// If the variable is a reference to another scope, the variable is removed from that
    /// scope as well.
    ///
    /// Note: it's irrelevant whether the variable is a scalar or array; it's going away.
    pub fn unset(&mut self, name: &str) {
        self.unset_at(self.current(), name, false);
    }

    /// Unset a variable at a given level in the stack.  If the variable at that level
    /// is linked to a higher level, follows the chain down, unsetting as it goes.
    fn unset_at(&mut self, level: usize, name: &str, array_only: bool) {
        // FIRST, if the variable at this level links to a lower level, follow the chain.
        if let Some(Var::Upvar(at)) = self.stack[level].map.get(name) {
            // NOTE: Using the variable true_level prevents a "doubly-borrowed" error.
            // Once Polonius is in use, this should no longer be necessary.
            let true_level = *at;
            self.unset_at(true_level, name, array_only);
        }

        // NEXT, remove the variable at this level.
        if array_only {
            if let Some(Var::Array(_)) = self.stack[level].map.get(name) {
                self.stack[level].map.remove(name);
            }
        } else {
            self.stack[level].map.remove(name);
        }
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
        self.stack[top].map.insert(name.into(), Var::Upvar(level));
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

    /// Gets a list of the names of the variables defined in the current scope.
    pub fn vars_in_scope(&self) -> MoltList {
        self.stack[self.current()]
            .map
            .keys()
            .cloned()
            .map(|x| Value::from(&x))
            .collect()
    }

    /// Determines whether the name names an array variable or not.
    pub fn array_exists(&self, name: &str) -> bool {
        match self.var(self.current(), name) {
            Some(Var::Array(_)) => true,
            _ => false,
        }
    }

    /// Gets a list of the array indices for the named array.  Returns the empty list
    /// if `name` doesn't name an array variable.
    pub fn array_indices(&self, name: &str) -> MoltList {
        match self.var(self.current(), name) {
            Some(Var::Array(map)) => map.keys().cloned().map(|x| Value::from(&x)).collect(),
            _ => Vec::new(),
        }
    }

    /// Gets the size of the named array.  Returns 0 if `name` doesn't name an array variable.
    pub fn array_size(&self, name: &str) -> usize {
        match self.var(self.current(), name) {
            Some(Var::Array(map)) => map.len(),
            _ => 0,
        }
    }

    /// Gets the content of an array as a flat list of names and values.  If the named
    /// variable is not an array, returns the empty list.
    pub fn array_get(&self, name: &str) -> MoltList {
        match self.var(self.current(), name) {
            Some(Var::Array(map)) => {
                let mut list = Vec::new();

                for (key,value) in map {
                    list.push(Value::from(key));
                    list.push(value.clone());
                }
                list
            }
            _ => Vec::new(),
        }
    }

    /// Unsets the value of the indexed array element in the current scope, if it exists.
    /// Does nothing if the array element doesn't exist, or the variable isn't an array
    /// variable.
    pub fn unset_element(&mut self, name: &str, index: &str) {
        if let Some(Var::Array(map)) = self.var_mut(self.current(), name) {
            map.remove(index);
        }
    }

    /// Unsets an array variable in the current scope, i.e., removes it from the scope.
    /// If the variable is a reference to another scope, the variable is removed from that
    /// scope as well.
    ///
    /// Only affects array variables.
    pub fn array_unset(&mut self, name: &str) {
        self.unset_at(self.current(), name, true);
    }

    //--------------------------------------------------------------
    // Utilities

    /// Retrieves an immutable borrow of the variable of the given name, searching the
    /// the scope stack for the variable starting at the current level and following the
    /// alias chain as needed.
    ///
    /// This call is the basis for all public APIs that retrieve information about a variable.
    ///
    /// TODO: Try using a loop rather than recursion, and see if that's any faster.
    fn var(&self, level: usize, name: &str) -> Option<&Var> {
        let var = self.stack[level].map.get(name);
        if let Some(Var::Upvar(at)) = var {
            self.var(*at, name)
        } else {
            var
        }
    }

    /// Retrieves an mutable borrow of the variable of the given name, searching the
    /// the scope stack for the variable starting at the current level and following the
    /// alias chain as needed.  If the variable is not found, creates a `Var::New` in the
    /// relevant place.  The caller should *always* replace the `Var::New` with either
    /// `Var::Scalar` or `Var::Array`.
    ///
    /// TODO: Try using a loop rather than recursion, and see if that's any faster.
    fn var_mut(&mut self, level: usize, name: &str) -> Option<&mut Var> {
        let var = self.stack[level].map.entry(name.into()).or_insert(Var::New);

        // NOTE: 11/28/2019.  Without this transmutation, the borrow checker will not allow the
        // recursive call to var_mut, even though it can be seen that all we are using
        // from the first borrow is the alias level. Under Polonius, a new borrow checker
        // currently under development, this pattern is allowed, and the unsafe code can
        // be deleted.
        let var: Option<&mut Var> = unsafe { ::core::mem::transmute(var) };

        if let Some(Var::Upvar(at)) = var {
            self.var_mut(*at, name)
        } else {
            var
        }
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
        let out = ss.get("a").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "1");

        let _ = ss.set("b", Value::from("2"));
        let out = ss.get("b").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "2");

        assert!(ss.get("c").unwrap().is_none());
    }

    #[test]
    fn test_set_get_elem_basic() {
        let mut ss = ScopeStack::new();

        // Set/get an element in an array
        let _ = ss.set_elem("a", "x", Value::from("1"));
        let out = ss.get_elem("a", "x").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "1");

        // Set/get another element in the same array
        let _ = ss.set_elem("a", "y", Value::from("2"));
        let out = ss.get_elem("a", "y").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "2");

        // Set/get an element in different array
        let _ = ss.set_elem("b", "x", Value::from("3"));
        let out = ss.get_elem("b", "x").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "3");

        // Fail to get an element from an existing array
        assert!(ss.get_elem("a", "z").unwrap().is_none());

        // Fail to get an element from an unknown variable
        assert!(ss.get_elem("c", "z").unwrap().is_none());
    }

    #[test]
    fn test_set_get_but_wrong_type() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::empty());
        let _ = ss.set_elem("b", "1", Value::empty());

        assert_eq!(
            ss.set("b", Value::empty()),
            molt_err!("can't set \"b\": variable is array")
        );
        assert_eq!(
            ss.set_elem("a", "1", Value::empty()),
            molt_err!("can't set \"a(1)\": variable isn't array")
        );

        assert_eq!(
            ss.get("b"),
            molt_err!("can't read \"b\": variable is array")
        );
        assert_eq!(
            ss.get_elem("a", "1"),
            molt_err!("can't read \"a(1)\": variable isn't array")
        );
    }

    #[test]
    fn test_unset_basic() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        assert!(ss.get("a").unwrap().is_some());
        ss.unset("a");
        assert!(ss.get("a").unwrap().is_none());
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
        assert!(ss.get("a").unwrap().is_none());
        assert!(ss.get("b").unwrap().is_none());
        assert!(ss.get("c").unwrap().is_none());

        let _ = ss.set("a", Value::from("3"));
        let _ = ss.set("b", Value::from("4"));
        let _ = ss.set("c", Value::from("5"));
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().unwrap().as_str(), "4");
        assert_eq!(ss.get("c").unwrap().unwrap().as_str(), "5");

        ss.pop();
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "1");
        assert_eq!(ss.get("b").unwrap().unwrap().as_str(), "2");
        assert!(ss.get("c").unwrap().is_none());
    }

    #[test]
    fn test_set_get_upvar() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", Value::from("1"));
        let _ = ss.set("b", Value::from("2"));

        ss.push();
        ss.upvar(0, "a");
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "1");
        assert!(ss.get("b").unwrap().is_none());

        let _ = ss.set("a", Value::from("3"));
        let _ = ss.set("b", Value::from("4"));
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().unwrap().as_str(), "4");

        ss.pop();
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "3");
        assert_eq!(ss.get("b").unwrap().unwrap().as_str(), "2");
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
        assert_eq!(ss.get("a").unwrap().unwrap().as_str(), "1");
        assert_eq!(ss.get("b").unwrap().unwrap().as_str(), "2");
    }

    #[test]
    fn test_unset_upvar() {
        let mut ss = ScopeStack::new();

        // Set a value at level 0
        let _ = ss.set("a", Value::from("1"));
        assert!(ss.get("a").unwrap().is_some());
        ss.push();
        assert!(ss.get("a").unwrap().is_none());

        // Link a@1 to a@0
        ss.upvar(0, "a");
        assert!(ss.get("a").unwrap().is_some());

        // Unset it; it should be unset in both scopes.
        ss.unset("a");

        assert!(ss.get("a").unwrap().is_none());
        ss.pop();
        assert!(ss.get("a").unwrap().is_none());
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

    #[test]
    fn test_global() {
        // Verify that we can upvar to a variable that doesn't yet exist.
        // Check both scalars and array elements.
        let mut ss = ScopeStack::new();

        ss.push();
        ss.upvar(0, "a");
        ss.upvar(0, "b");
        let _ = dbg!(ss.set("a", Value::from("1")));
        let _ = dbg!(ss.set_elem("b", "1", Value::from("2")));
        ss.pop();

        let out = ss.get("a").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "1");

        let out = ss.get_elem("b", "1").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "2");
    }

    #[test]
    fn test_array_indices() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", "zero".into());
        let _ = ss.set_elem("b", "1", "one".into());
        let _ = ss.set_elem("b", "2", "two".into());

        assert_eq!(ss.array_indices("x"), Vec::new());
        assert_eq!(ss.array_indices("a"), Vec::new());

        let list = ss.array_indices("b");
        assert!(list.len() == 2);
        assert!(list.contains(&"1".into()));
        assert!(list.contains(&"2".into()));
    }

    #[test]
    fn test_array_size() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", "zero".into());
        let _ = ss.set_elem("b", "1", "one".into());
        let _ = ss.set_elem("b", "2", "two".into());

        assert_eq!(ss.array_size("x"), 0);
        assert_eq!(ss.array_size("a"), 0);
        assert_eq!(ss.array_size("b"), 2);
    }

    #[test]
    fn test_array_get() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", "zero".into());
        let _ = ss.set_elem("b", "1", "one".into());
        let _ = ss.set_elem("b", "2", "two".into());

        assert_eq!(ss.array_get("x"), Vec::new());
        assert_eq!(ss.array_get("a"), Vec::new());

        let list = ss.array_get("b");
        assert!(list.len() == 4);
        assert!(list.contains(&"1".into()));
        assert!(list.contains(&"one".into()));
        assert!(list.contains(&"2".into()));
        assert!(list.contains(&"two".into()));
    }

    #[test]
    fn test_unset_element() {
        let mut ss = ScopeStack::new();

        let _ = ss.set("a", "zero".into());
        let _ = ss.set_elem("b", "1", "one".into());
        let _ = ss.set_elem("b", "2", "two".into());

        // Array unset of an unknown variable has no effect.
        ss.unset_element("x", "1"); // No error

        // Array unset of a scalar has no effect.
        ss.unset_element("a", "1");
        let out = ss.get("a").unwrap();
        assert!(out.is_some());
        assert_eq!(out.unwrap().as_str(), "zero");

        // Array unset of an element unsets just that element.
        ss.unset_element("b", "1");
        assert!(ss.get_elem("b", "1").unwrap().is_none());
        assert!(ss.get_elem("b", "2").unwrap().is_some());
    }
}
