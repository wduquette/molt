# dict *subcommand* ?*arg* ...?

This command manipulates TCL dictionaries.  A dictionary is a Molt value containing a hash map
from keys to values.  As usual with hash maps, the order of keys is undefined.

| Subcommand                            | Description                                  |
| ------------------------------------- | -------------------------------------------- |
| [dict create](#dict-create-key-value) | Creates a dictionary                         |

**TCL Liens**

* Not all of the standard TCL `dict` subcommands are implemented at this time.
* `dict keys` and `dict values` do not support filtering using glob or regex matches
   at this time.  The plan is to support glob and regex matching as an optional feature.

## dict create ?*key* *value*...?

Creates a dictionary given any number of key/value pairs.

```tcl
% set dict [dict create a 1 b 2]
a 1 b 2
% dict get $dict a
1
```
