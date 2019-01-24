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
pub struct Scope {
    map: HashMap<String,Var>
}

impl Scope {
    pub fn new() -> Self {
        Scope { map: HashMap::new() }
    }
}

#[derive(Default)]
pub struct VarStack {
    stack: Vec<Scope>,
}

impl VarStack {
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

    /// Links a variable in the current scope to variable at the given level, counting
    /// from 0, the global scope.
    pub fn upvar(&mut self, name: &str, level: usize) {
        assert!(level < self.top(), "Can't upvar to current stack level");
        self.stack[level].map.insert(name.into(), Var::Level(level));
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
}
