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

use std::collections::HashMap;

/// A variable in a `Scope`.  If the variable is defined in the `Scope`, it has a
/// `Value`; if it is a reference to a variable in a higher scope (e.g., a global) then
/// the `Level` gives the referenced scope.
enum Var {
    Value(String),
    Level(usize)
}

/// A scope: a level in the `ScopeStack`.  It contains a hash table of `Var`'s by name.
#[derive(Default)]
struct Scope {
    /// Vars in this scope by name.
    map: HashMap<String,Var>
}

impl Scope {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Scope { map: HashMap::new() }
    }
}

/// The scope stack: a stack of variable scopes corresponding to the Molt `proc`
/// call stack.
#[derive(Default)]
pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
    /// Create a scope stack containing only scope `0`, the global scope.
    pub fn new() -> Self {
        let mut vs = Self {
            stack: Vec::new(),
        };

        vs.stack.push(Scope::new());

        vs
    }

    /// Sets a variable to a value in the current scope.  If the variable is linked to
    /// another scope, the value is set there instead.
    pub fn set(&mut self, name: &str, value: &str) -> String {
        let top = self.stack.len() - 1;

        self.set_at(top, name, value);
        value.into()
    }

    /// Unsets a variable in the current scope, i.e., removes it from the scope.
    /// If the variable is a reference to another scope, the variable is removed from that
    /// scope as well.
    pub fn unset(&mut self, name: &str) {
        let top = self.stack.len() - 1;
        self.unset_at(top, name);
    }

    /// Gets the value of the named variable in the current scope, if present.
    pub fn get(&self, name: &str) -> Option<String> {
        let top = self.stack.len() - 1;

        self.get_at(top, name)
    }

    /// Gets the value at the given level, recursing up the stack as needed.
    fn get_at(&self, level: usize, name: &str) -> Option<String> {
        match self.stack[level].map.get(name) {
            Some(Var::Value(value)) => Some(value.clone()),
            Some(Var::Level(at)) => self.get_at(*at, name),
            _ =>  None,
        }
    }

    /// Set a variable to a value at a given level in the stack.  If the variable at that level
    /// is linked to a higher level, sets it at that level instead.
    fn set_at(&mut self, level:usize, name: &str, value: &str) {
        match self.stack[level].map.get(name) {
            Some(Var::Level(at)) => {
                self.set_at(*at, name, value);
            }
            _ => {
                self.stack[level].map.insert(name.into(), Var::Value(value.into()));
            }
        }
    }

    /// Unset a variable at a given level in the stack.  If the variable at that level
    /// is linked to a higher level, follows the chain down, unsetting as it goes.
    fn unset_at(&mut self, level: usize, name: &str) {
        // FIRST, if the variable at this level links to a lower level, follow the chain.
        if let Some(Var::Level(at)) = self.stack[level].map.get(name) {
            self.unset_at(*at, name);
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
    /// TODO: Should return a MoltList.
    pub fn vars_in_scope(&self) -> Vec<String> {
        let top = self.stack.len() - 1;
        let vec: Vec<String> = self.stack[top].map.keys().cloned().collect();

        vec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vs = ScopeStack::new();
        assert_eq!(vs.stack.len(), 1);
    }

    #[test]
    fn test_push() {
        let mut vs = ScopeStack::new();
        vs.push();
        assert_eq!(vs.stack.len(), 2);
        vs.push();
        assert_eq!(vs.stack.len(), 3);
    }

    #[test]
    fn test_pop() {
        let mut vs = ScopeStack::new();
        vs.push();
        vs.push();
        assert_eq!(vs.stack.len(), 3);
        vs.pop();
        assert_eq!(vs.stack.len(), 2);
        vs.pop();
        assert_eq!(vs.stack.len(), 1);
    }

    #[test]
    #[should_panic]
    fn test_pop_global_scope() {
        let mut vs = ScopeStack::new();
        assert_eq!(vs.stack.len(), 1);
        vs.pop();
    }

    #[test]
    fn test_current() {
        let mut vs = ScopeStack::new();
        assert_eq!(vs.current(), 0);
        vs.push();
        assert_eq!(vs.current(), 1);
        vs.push();
        assert_eq!(vs.current(), 2);
        vs.pop();
        assert_eq!(vs.current(), 1);
        vs.pop();
        assert_eq!(vs.current(), 0);
    }

    #[test]
    fn test_set_get() {
        let mut vs = ScopeStack::new();

        vs.set("a", "1");
        assert_eq!(vs.get("a"), Some("1".into()));

        vs.set("b", "2");
        assert_eq!(vs.get("b"), Some("2".into()));

        assert_eq!(vs.get("c"), None);
    }

    #[test]
    fn test_set_levels() {
        let mut vs = ScopeStack::new();

        vs.set("a", "1");
        vs.set("b", "2");

        vs.push();
        assert_eq!(vs.get("a"), None);
        assert_eq!(vs.get("b"), None);
        assert_eq!(vs.get("c"), None);

        vs.set("a", "3");
        vs.set("b", "4");
        vs.set("c", "5");
        assert_eq!(vs.get("a"), Some("3".into()));
        assert_eq!(vs.get("b"), Some("4".into()));
        assert_eq!(vs.get("c"), Some("5".into()));

        vs.pop();
        assert_eq!(vs.get("a"), Some("1".into()));
        assert_eq!(vs.get("b"), Some("2".into()));
        assert_eq!(vs.get("c"), None);
    }

    #[test]
    fn test_set_get_upvar() {
        let mut vs = ScopeStack::new();

        vs.set("a", "1");
        vs.set("b", "2");

        vs.push();
        vs.upvar(0, "a");
        assert_eq!(vs.get("a"), Some("1".into()));
        assert_eq!(vs.get("b"), None);

        vs.set("a", "3");
        vs.set("b", "4");
        assert_eq!(vs.get("a"), Some("3".into()));
        assert_eq!(vs.get("b"), Some("4".into()));

        vs.pop();
        assert_eq!(vs.get("a"), Some("3".into()));
        assert_eq!(vs.get("b"), Some("2".into()));
    }

    #[test]
    fn test_unset() {
        let mut vs = ScopeStack::new();

        vs.set("a", "1");
        assert_eq!(vs.get("a"), Some("1".into()));
        vs.unset("a");
        assert_eq!(vs.get("a"), None);
    }

    #[test]
    fn test_unset_levels() {
        let mut vs = ScopeStack::new();

        vs.set("a", "1");
        vs.set("b", "2");

        vs.push();
        vs.set("a", "3");

        vs.unset("a");  // Was set in this scope
        vs.unset("b");  // Was not set in this scope

        vs.pop();
        assert_eq!(vs.get("a"), Some("1".into()));
        assert_eq!(vs.get("b"), Some("2".into()));
    }

    #[test]
    fn test_unset_upvar() {
        let mut vs = ScopeStack::new();

        // Set a value at level 0
        vs.set("a", "1");
        vs.push();

        // Link a@1 to a@0
        vs.upvar(0, "a");

        // Unset it; it should be unset in both scopes.
        vs.unset("a");

        assert_eq!(vs.get("a"), None);
        vs.pop();
        assert_eq!(vs.get("a"), None);
    }
}
