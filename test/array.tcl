# Test Script: array command.

test array-1.1 {array names, no var} {
    array names
} -error {wrong # args: should be "array names arrayName"}

test array-1.2 {array names, unknown var} {
    array names unknown_variable
} -ok {}

test array-1.3 {array names, scalar var} {
    set scalar 1
    array names scalar
} -ok {}

test array-1.4 {array names, array var} {
    set a(1) one
    set a(2) two
    #  Really need to [lsort] the list of names, but I don't have [lsort] yet.
    #  In the meantime, just check the length.
    llength [array names a]
} -ok {2}

test array-2.1 {array size, no var} {
    array size
} -error {wrong # args: should be "array size arrayName"}

test array-2.2 {array size, unknown var} {
    array size unknown_variable
} -ok {0}

test array-2.3 {array size, scalar var} {
    set scalar 1
    array size scalar
} -ok {0}

test array-2.4 {array size, array var} {
    set a(1) one
    set a(2) two
    array size a
} -ok {2}
