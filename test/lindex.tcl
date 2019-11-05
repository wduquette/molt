# Test Script: lindex command.

test list-1.1 {index in range} {
    lindex {a b c} 1
} -ok {b}

test list-1.2 {index nested in range} {
    lindex {a {b c d} e} 1 2
} -ok {d}

test list-1.3 {index out of range} {
    lindex {a b c} 4
} -ok {}

test list-1.4 {index nested out of range} {
    lindex {a {b c d} e} 1 4
} -ok {}

test list-1.5 {index nested too deep} {
    lindex {a {b c d} e} 1 1 2
} -ok {}

test list-1.6 {no index} {
    lindex {a b c}
} -ok {a b c}

test list-2.1 {empty index list} {
    lindex {a b c} {}
} -ok {a b c}

test list-2.2 {index list} {
    lindex {a {b c d} e} {1 1}
} -ok {c}

test list-3.1 {no arguments} {
    lindex
} -error {wrong # args: should be "lindex list ?index ...?"}
