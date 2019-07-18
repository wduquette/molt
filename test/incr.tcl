# Test Suite: incr command

test incr-1.1 {incr command no args} {
    incr
} -error {wrong # args: should be "incr varName ?increment?"}

test incr-2.1 {incr new var} -body {
    incr a
} -cleanup {
    unset a
} -ok {1}

test incr-2.2 {incr existing var} -body {
    set a 5
    incr a
} -cleanup {
    unset a
} -ok {6}

test incr-2.3 {var is set} -body {
    incr a
    set a
} -cleanup {
    unset a
} -ok {1}

test incr-2.4 {increment can be specified} -body {
    set a 5
    incr a 7
    set a
} -cleanup {
    unset a
} -ok {12}
