# [expr] command testing.

# lexpr expr...
#
# Evaluates each expression and returns a list of the results.
proc lexpr {args} {
    foreach expr $args {
        lappend result [expr $expr]
    }

    return $result
}

# expr-1.*: Literals

test expr-1.1 {
    lexpr {1} {-1} {+1}
} -ok {1 -1 1}

test expr-1.2 {
    lexpr {1.1} {-1.1} {+1.1} {1.1e3} {-1.1e3} {1.1e-3}
} -ok {1.1 -1.1 1.1 1100 -1100 0.0011}

test expr-1.3 {
    lexpr {1.0} {-1.0} {+1.0}
    # In Tcl, would be "1.0", etc.
    # TODO: Look into floating point compatibility with TCL.
} -ok {1 -1 1}

# test expr-1.4 {
#    lexpr true yes on false no off
# } -ok {true yes on false no off}

# expr-2.*: arithmetic

test expr-2.1 {
    lexpr {1 + 2} {3 + 2} {3 + 0}
} -ok {3 5 3}

test expr-2.2 {
    lexpr {3 - 1} {1 - 3} {3 - 0}
} -ok {2 -2 3}

test expr-2.3 {
    lexpr {2 * 3} {0 * 2} {2 * 0}
} -ok {6 0 0}

test expr-2.4 {
    lexpr {4 / 2} {5 / 2} {6 / 2}
} -ok {2 2 3}

test expr-2.5 {
    expr {2 / 0}
} -error {divide by zero}

test expr-2.6 {
    lexpr {1 % 4} {3 % 4} {5 % 3}
} -ok {1 3 2}

test expr-2.7 {
    lexpr {1.1 + 2} {3 + 2.1} {3 + 1.0}
} -ok {3.1 5.1 4}

test expr-2.8 {
    lexpr {3.1 - 1} {1.1 - 3} {3.1 - 0}
} -ok {2.1 -1.9 3.1}

test expr-2.9 {
    lexpr {2.5 * 3} {0.0 * 2} {2.0 * 0}
} -ok {7.5 0 0}

test expr-2.10 {
    lexpr {4.0 / 2} {5 / 2.0} {6.2 / 2}
} -ok {2 2.5 3.1}

test expr-2.11 {
    expr {2.1 / 0.0}
} -error {divide by zero}

test expr-2.12 {
    expr {2.2 % 0.0}
} -error {can't use floating-point value as operand of "%"}

# expr-3.*: Logical Operators
#
# Note: can't use boolean constants yet.

test expr-3.1 {
    lexpr {1 && 1} {1 && 0} {0 && 1} {0 && 0}
} -ok {1 0 0 0}

test expr-3.2 {
    lexpr {1 || 1} {1 || 0} {0 || 1} {0 || 0}
} -ok {1 1 1 0}

test expr-3.3 {
    lexpr {!1} {!0}
} -ok {0 1}

test expr-3.4 {
    lexpr {1.1 && 1.1} {1.1 && 0.0} {0.0 && 1.1} {0.0 && 0.0}
} -ok {1 0 0 0}

test expr-3.5 {
    lexpr {1.1 || 1.1} {1.1 || 0.0} {0.0 || 1.1} {0.0 || 0.0}
} -ok {1 1 1 0}

test expr-3.6 {
    lexpr {!1.1} {!0.0}
} -ok {0 1}

# expr-4.*: Comparisons

test expr-4.1 {
    lexpr {0 == 0} {1 == 0} {0 == 1}
} -ok {1 0 0}

test expr-4.2 {
    lexpr {0 != 0} {1 != 0} {0 != 1}
} -ok {0 1 1}

test expr-4.3 {
    lexpr {0 < 1} {1 < 1} {2 < 1}
} -ok {1 0 0}

test expr-4.4 {
    lexpr {0 <= 1} {1 <= 1} {2 <= 1}
} -ok {1 1 0}

test expr-4.5 {
    lexpr {0 > 1} {1 > 1} {2 > 1}
} -ok {0 0 1}

test expr-4.6 {
    lexpr {0 >= 1} {1 >= 1} {2 >= 1}
} -ok {0 1 1}

test expr-4.7 {
    lexpr {1.1 == 1.1} {1.1 != 1.1} {1.1 < 1.1} {1.1 <= 1.1} {1.1 > 1.1} {1.1 >= 1.1}
} -ok {1 0 0 1 0 1}

# expr-5.*: bitwise operators

test expr-5.1 {
    lexpr {1 & 2} {2 & 2} {3 & 2}
} -ok {0 2 2}

test expr-5.2 {
    lexpr {1 | 2} {2 | 2} {3 | 2}
} -ok {3 2 3}

test expr-5.3 {
    lexpr {~1} {~0}
} -ok {-2 -1}
