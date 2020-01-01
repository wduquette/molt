//! Dictionary Parsing and Formatting

use crate::list::list_to_string;
use crate::types::MoltDict;
use crate::types::MoltList;
use crate::value::Value;
use std::collections::HashMap;

/// Create an empty dict.
pub fn dict_create() -> MoltDict {
    HashMap::new()
}

/// Converts a dictionary into a string.
pub fn dict_to_string(dict: &MoltDict) -> String {
    let mut vec: MoltList = Vec::new();

    for (k, v) in dict {
        vec.push(k.clone());
        vec.push(v.clone());
    }

    list_to_string(&vec)
}

/// Converts a vector of values into a dictionary.  The list must have
/// an even number of elements.
pub fn list_to_dict(list: &[Value]) -> MoltDict {
    assert!(list.len() % 2 == 0);

    let mut dict = HashMap::new();

    for i in (0..list.len()).step_by(2) {
        dict.insert(list[i].clone(), list[i + 1].clone());
    }

    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MoltDict;
    use std::collections::HashMap;

    #[test]
    fn test_dict_to_string() {
        let mut dict: MoltDict = HashMap::new();

        assert_eq!(dict_to_string(&dict), "");

        dict.insert("abc".into(), "123".into());

        assert_eq!(dict_to_string(&dict), "abc 123");
    }
}
