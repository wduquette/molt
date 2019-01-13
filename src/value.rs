/// GCL Data Values
///
/// Cognate to a Tcl 8 Tcl_Obj.
/// For now this is just a wrapped String; ultimately, it will be
/// a two-legged stork, like Tcl_Obj.

/// A GCL data value

#[derive(Clone,Default)]
pub struct Value {
    str: String,
}

impl Value {
    pub fn new() -> Self {
        Self {
            str: String::new(),
        }
    }

    pub fn from(str: &str) -> Self {
        Self {
            str: str.into(),
        }
    }

    pub fn push_str(&mut self, text: &str) {
        self.str.push_str(text);
    }

    pub fn as_string(&self) -> String {
        self.str.clone()
    }

    pub fn clear(&mut self) {
        self.str.clear();
    }
}
