//! Dictionary Parsing and Formatting

use crate::types::MoltDict;
use crate::types::MoltList;
use crate::list::list_to_string;

pub fn dict_to_string(dict: &MoltDict) -> String {
    let mut vec: MoltList = Vec::new();

    for (k,v) in dict {
        vec.push(k.clone());
        vec.push(v.clone());
    }

    list_to_string(&vec)
}

#[cfg(test)]
mod tests {
    use crate::types::MoltDict;
    use std::collections::HashMap;
    use super::*;

    #[test]
    fn test_dict_to_string() {
        let mut dict: MoltDict = HashMap::new();

        assert_eq!(dict_to_string(&dict), "");

        dict.insert("abc".into(), "123".into());

        assert_eq!(dict_to_string(&dict), "abc 123");
    }
}
