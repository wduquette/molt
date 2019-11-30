# array *subcommand* ?*arg* ...?

This command queries and manipulates array variables.

* [array names](#array-names)
* [array size](#array-size)

## array names *arrayName*

Returns an unsorted list of the indices of the named array variable.  If there is no array
variable with the given name, returns the empty list.

**TCL Liens**: does not support filtering the list.

## array size *arrayName*

Returns the number of elements in the named array.  If there is no array
variable with the given name, returns "0".
