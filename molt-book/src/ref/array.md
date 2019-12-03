# array *subcommand* ?*arg* ...?

This command queries and manipulates array variables.

* [array exists](#array-exists)
* [array get](#array-get)
* [array names](#array-names)
* [array size](#array-size)
* [array unset](#array-unset)

## array exists *arrayName*

Returns 1 if *arrayName* names an array variable, and 0 otherwise.

## array get *arrayName*

Returns a flat list of the keys and values in the named array.  The key/value pairs appear
in unsorted order. If there is no array variable with the given name, returns the empty list.

**TCL Liens**: does not support filtering the list.

## array names *arrayName*

Returns an unsorted list of the indices of the named array variable.  If there is no array
variable with the given name, returns the empty list.

**TCL Liens**: does not support filtering the list.

## array set *arrayName* *list*

Merges a flat list of keys and values into the array, creating the array variable if necessary.
The list must have an even number of elements.  It's an error if the variable exists but isn't
an array.

## array size *arrayName*

Returns the number of elements in the named array.  If there is no array
variable with the given name, returns "0".

## array unset *arrayName* ?*index*?

Unsets the array element in *arrayName* with the given *index*.  If index is not given,
unsets the entire array.

Note:

* `array unset my_array` is equivalent to `unset my_array`.
* `array unset my_array my_index` is equivalent to `unset my_array(my_index)`

The real value of `array unset` depends on pattern matching on the index argument, which is
not yet available.

**TCL Liens**: does not support glob matching on the optional argument.
