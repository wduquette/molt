//! Variable Scope
//!
//! A VarScope holds the variables for a scope in the call stack
//!
//! Issue: we need to be able to map a variable in a higher level to a variable in
//! another level (i.e., the global scope).  I think the answer is to use a RefCell, an
//! immutable reference to a mutable string.  We'll see.

use std::collections::HashMap;

enum Var {
    Value(String),
    Level(usize)
}

#[derive(Default)]
struct Scope {
    map: HashMap<String,Var>
}

impl Scope {
    pub fn new() -> Self {
        Scope { map: HashMap::new() }
    }
}

#[derive(Default)]
pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
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

    pub fn unset(&mut self, name: &str) {
        let top = self.stack.len() - 1;
        // self.stack[top].map.remove(name);
        self.unset_at(top, name);
    }

    /// Gets the value of the named variable in the current scope.
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
    /// is linked to a previous level, sets it at that level instead.
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
    /// is linked to a previous level, follows the chain down, unsetting as it goes.
    fn unset_at(&mut self, level: usize, name: &str) {
        // FIRST, if the variable at this level links to a lower level, follow the chain.
        if let Some(Var::Level(at)) = self.stack[level].map.get(name) {
            self.unset_at(*at, name);
        }

        // NEXT, remove the link at this level.
        self.stack[level].map.remove(name);
    }

    /// Links a variable in the current scope to variable at the given level, counting
    /// from 0, the global scope.
    pub fn upvar(&mut self, level: usize, name: &str) {
        assert!(level < self.top(), "Can't upvar to current stack level");
        let top = self.top();
        self.stack[top].map.insert(name.into(), Var::Level(level));
    }

    pub fn top(&self) -> usize {
        self.stack.len() - 1
    }

    /// Push a new scope onto the stack.
    pub fn push(&mut self) {
        self.stack.push(Scope::new());
    }

    /// Pop the top scope from the stack. Panics if we're at the global scope.
    pub fn pop(&mut self) {
        self.stack.pop();
        assert!(!self.stack.is_empty(), "Popped global scope!");
    }

    pub fn get_visible_names(&self) -> Vec<String> {
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
    fn test_top() {
        let mut vs = ScopeStack::new();
        assert_eq!(vs.top(), 0);
        vs.push();
        assert_eq!(vs.top(), 1);
        vs.push();
        assert_eq!(vs.top(), 2);
        vs.pop();
        assert_eq!(vs.top(), 1);
        vs.pop();
        assert_eq!(vs.top(), 0);
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
