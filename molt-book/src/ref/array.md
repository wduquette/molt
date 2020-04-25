# array -- Query and manipulate array variables

**Syntax: array *subcommand* ?*arg* ...?**

This command queries and manipulates array variables.

| Subcommand                    | Description                                    |
| ----------------------------- | ---------------------------------------------- |
| [array exists](#array-exists) | Is the string the name of an array variable?   |
| [array get](#array-get)       | A dictionary of the array's elements by name   |
| [array names](#array-names)   | A list of the array's indices                  |
| [array set](#array-set)       | Merges a dictionary of elements into the array |
| [array size](#array-size)     | The number of elements in the array            |
| [array unset](#array-unset)   | Unsets an array variable                       |

**TCL Liens**

* Does not support filtering using glob or regex matches at this time.  The plan is to
  support glob and regex matching as a configuration option at build time.
* Will never support the array iteration commands `array startsearch`, `array anymore`,
  `array donesearch`, `array nextelement`, because they are unnecessary and rarely used.
  The normal idiom for iterating over an array is a `foreach` over `array names`.
* Will never support `array statistics`, as Rust's `std::collections::HashMap` doesn't
  provide a way to gather them.

## array exists

**Syntax: array exists *arrayName***

Returns 1 if *arrayName* names an array variable, and 0 otherwise.

## array get

**Syntax: array get *arrayName***

Returns a flat list of the keys and values in the named array.  The key/value pairs appear
in unsorted order. If there is no array variable with the given name, returns the empty list.

**TCL Liens**: does not support filtering the list using glob and regex matches.

## array names

**Syntax: array names *arrayName***

Returns an unsorted list of the indices of the named array variable.  If there is no array
variable with the given name, returns the empty list.

**TCL Liens**: does not support filtering the list using glob and regex matches.

## array set

**Syntax: array set *arrayName* *list***

Merges a flat list of keys and values into the array, creating the array variable if necessary.
The list must have an even number of elements.  It's an error if the variable exists but has
a scalar value, or if *arrayName* names an array element.

## array size

**Syntax: array size *arrayName***

Returns the number of elements in the named array.  If there is no array
variable with the given name, returns "0".

## array unset

**Syntax: array unset *arrayName* ?*index*?**

Unsets the array element in *arrayName* with the given *index*.  If index is not given,
unsets the entire array.

Note:

* `array unset my_array` is equivalent to `unset my_array`, but only works on array variables.
* `array unset my_array my_index` is equivalent to `unset my_array(my_index)`

The real value of `array unset` depends on pattern matching on the index argument, which is
not yet available.

**TCL Liens**: does not support glob matching on the optional argument.
