//! Dictionary Utilities
//!
//! Dictionaries are implemented on top of the indexmap::IndexMap type, a drop-in replacement
//! for std::collections::HashMap that preserves the order of keys.
//!
//! * Warning: when removing items from a dictionary, use `dict_remove`, defined here, as it
//!   preserves the order.  Using `IndexMap::remove` does not.

use crate::list::list_to_string;
use crate::molt_ok;
use crate::types::MoltDict;
use crate::types::MoltList;
use crate::types::MoltResult;
use crate::value::Value;
use indexmap::IndexMap;

/// Create an empty dict.
pub fn dict_new() -> MoltDict {
    IndexMap::new()
}

/// Inserts a key and value into a copy of the dictionary, returning the new dictionary.
pub(crate) fn dict_insert(dict: &MoltDict, key: &Value, value: &Value) -> MoltDict {
    let mut new_dict = dict.clone();
    new_dict.insert(key.clone(), value.clone());
    new_dict
}

/// Given a Value containing a dictionary, a list of keys, and a value,
/// inserts the value into the (possibly nested) dictionary, returning the new
/// dictionary value.
pub(crate) fn dict_path_insert(dict_val: &Value, keys: &[Value], value: &Value) -> MoltResult {
    assert!(!keys.is_empty());

    let dict = dict_val.as_dict()?;

    if keys.len() == 1 {
        molt_ok!(dict_insert(&*dict, &keys[0], &value))
    } else if let Some(dval) = dict.get(&keys[0]) {
        molt_ok!(dict_insert(
            &*dict,
            &keys[0],
            &dict_path_insert(dval, &keys[1..], value)?
        ))
    } else {
        let dval = Value::from(dict_new());
        molt_ok!(dict_insert(
            &*dict,
            &keys[0],
            &dict_path_insert(&dval, &keys[1..], value)?
        ))
    }
}

/// Converts a dictionary into a string.
pub(crate) fn dict_to_string(dict: &MoltDict) -> String {
    let mut vec: MoltList = Vec::new();

    for (k, v) in dict {
        vec.push(k.clone());
        vec.push(v.clone());
    }

    list_to_string(&vec)
}

/// Converts a vector of values into a dictionary.  The list must have
/// an even number of elements.
pub(crate) fn list_to_dict(list: &[Value]) -> MoltDict {
    assert!(list.len() % 2 == 0);

    let mut dict = dict_new();

    for i in (0..list.len()).step_by(2) {
        dict.insert(list[i].clone(), list[i + 1].clone());
    }

    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MoltDict;

    #[test]
    fn test_dict_to_string() {
        let mut dict: MoltDict = dict_new();

        assert_eq!(dict_to_string(&dict), "");

        dict.insert("abc".into(), "123".into());

        assert_eq!(dict_to_string(&dict), "abc 123");
    }
}
