# dict *subcommand* ?*arg* ...?

This command manipulates TCL dictionaries.  A dictionary is a Molt value containing a hash map
from keys to values.  As usual with hash maps, the order of keys is undefined.

| Subcommand                                      | Description                                  |
| ----------------------------------------------- | -------------------------------------------- |
| [dict create](#dict-create-key-value-)          | Creates a dictionary                         |
| [dict exists](#dict-exists-dictionary-key-key-) | Is there a value with these keys?            |
| [dict get](#dict-get-dictionary-key-)           | Gets a value from the dictionary             |
| [dict get](#dict-get-dictvarname-key-key-value) | Sets a value in a dictionary                 |
| [dict size](#dict-size-dictionary)              | The number of elements in the dictionary     |

**TCL Liens**

* Not all of the standard TCL `dict` subcommands are implemented at this time.
* `dict keys` and `dict values` do not support filtering using glob or regex matches
   at this time.  The plan is to support glob and regex matching as an optional feature.
* `dict info` is not supported; it is intended for tuning the standard TCL hash table
  implementation.  Molt relies on `std::collections::HashMap`.

## dict create ?*key* *value* ...?

Creates a dictionary given any number of key/value pairs.

```tcl
% set dict [dict create a 1 b 2]
a 1 b 2
% dict get $dict a
1
```

## dict exists *dictionary* *key* ?*key* ...?

Returns 1 if the *key* (or the path of keys through nested dictionaries) is found in the
given *dictionary* value, and 0 otherwise.  It returns 1 exactly when `dict get` will
succeed for the same arguments.  It does not throw errors on invalid dictionary values, but
simply returns 0.

Looks up the *key* in the *dictionary* and returns its value.  It's an error if the *key* is
not present in the dictionary.  If multiple keys are provided, the command looks up values
through nested dictionaries.  If no keys are provided, the dictionary itself is returned.

```tcl
% dict exists {a 1 b 2} b
1
% dict exists {a {x 1 y2} b {p 3 q 4}} b p
1
% dict exists {a 1 b 2} c
0
% dict exists not-a-dict a
0
```

## dict get *dictionary* ?*key* ...?

Looks up the *key* in the *dictionary* and returns its value.  It's an error if the *key* is
not present in the dictionary.  If multiple keys are provided, the command looks up values
through nested dictionaries.  If no keys are provided, the dictionary itself is returned.

```tcl
% dict get {a 1 b 2} b
2
% dict get {a {x 1 y2} b {p 3 q 4}} b p
3
```

## dict set *dictVarName* *key* ?*key* ...? *value*

Given the name of a variable containing a dictionary, sets the *value* of the given *key* in
the dictionary. If multiple keys are given, the command indexes down the path of keys and sets
the value in the nested dictionary.  The variable is created if it does not exist, and the nested
dictionaries are also created as needed.  Returns the modified dictionary, which is also saved
back into the variable.

```tcl
% dict set var a 1
a 1
% dict set var b 2
a 1 b 2
% dict set var c x 3
a 1 b 2 c {x 3}
% dict set var c y z 4
a 1 b 2 c {x 3 y {z 4}}
```

## dict size *dictionary*

Gets the number of entries in the *dictionary*.

```
% set dict [dict create a 1 b 2 c 3]
a 1 b 2 c 3
% dict size $dict
3
```
