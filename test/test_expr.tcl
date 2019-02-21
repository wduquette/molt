test expr-1.1 {
    list [expr {1}] [expr {-1}] [expr {+1}]
} -ok {1 -1 1}

test expr-1.2 {
    list [expr {1.1}] [expr {-1.1}] [expr {+1.1}]
} -ok {1.1 -1.1 1.1}

test expr-1.3 {
    list [expr {1.0}] [expr {-1.0}] [expr {+1.0}]
    # In Tcl, would be "1.0", etc.
} -ok {1 -1 1}

test expr-2.1 {
    list [expr {1 + 2}] [expr {3 + 2}] [expr {3 + 0}]
} -ok {3 5 3}

test expr-2.2 {
    list [expr {3 - 1}] [expr {1 - 3}] [expr {3 - 0}]
} -ok {2 -2 3}

test expr-2.3 {
    list [expr {2 * 3}] [expr {0 * 2}] [expr {2 * 0}]
} -ok {6 0 0}

test expr-2.4 {
    list [expr {4 / 2}] [expr {5 / 2}] [expr {6 / 2}]
} -ok {2 2 3}

test expr-2.5 {
    expr {2 / 0}
} -error {divide by zero}

test expr-2.6 {
    list [expr {1 % 4}] [expr {3 % 4}] [expr {5 % 3}]
} -ok {1 3 2}

test expr-3.1 {
    list [expr {1 && 1}] [expr {1 && 0}] [expr {0 && 1}] [expr {0 && 0}]
} -ok {1 0 0 0}

test expr-3.2 {
    list [expr {1 || 1}] [expr {1 || 0}] [expr {0 || 1}] [expr {0 || 0}]
} -ok {1 1 1 0}

test expr-3.3 {
    list [expr {!1}] [expr {!0}]
} -ok {0 1}

test expr-3.4 {
    # Not gonna work until I handle bare words?
    # expr {true || 1}
} -ok {}

test expr-4.1 {
    list [expr {0 == 0}] [expr {1 == 0}] [expr {0 == 1}]
} -ok {1 0 0}

test expr-4.2 {
    list [expr {0 != 0}] [expr {1 != 0}] [expr {0 != 1}]
} -ok {0 1 1}

test expr-4.3 {
    list [expr {0 < 1}] [expr {1 < 1}] [expr {2 < 1}]
} -ok {1 0 0}

test expr-4.4 {
    list [expr {0 <= 1}] [expr {1 <= 1}] [expr {2 <= 1}]
} -ok {1 1 0}

test expr-4.5 {
    list [expr {0 > 1}] [expr {1 > 1}] [expr {2 > 1}]
} -ok {0 0 1}

test expr-4.6 {
    list [expr {0 >= 1}] [expr {1 >= 1}] [expr {2 >= 1}]
} -ok {0 1 1}

test expr-5.1 {
    list [expr {1 & 2}] [expr {2 & 2}] [expr {3 & 2}]
} -ok {0 2 2}

test expr-5.2 {
    list [expr {1 | 2}] [expr {2 | 2}] [expr {3 | 2}]
} -ok {3 2 3}

test expr-5.3 {
    list [expr {~1}] [expr {~0}]
} -ok {-2 -1}
