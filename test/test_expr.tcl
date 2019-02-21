test expr-1.1 {
    expr {1}
} -ok {1}

test expr-1.2 {
    expr {1.1}
} -ok {1.1}

test expr-1.3 {
    expr {1.0}
    # Possibly, should be "1.0".
} -ok {1}

test expr-2.1 {
    expr {1 + 2}
} -ok {3}

test expr-2.2 {
    expr {3 - 2}
} -ok {1}

test expr-2.3 {
    expr {2 - 3}
} -ok {-1}

test expr-2.4 {
    expr {2 * 3}
} -ok {6}

test expr-2.5 {
    expr {4 / 2}
} -ok {2}

test expr-2.6 {
    expr {2 / 4}
} -ok {0}

test expr-2.7 {
    expr {1 % 4}
} -ok {1}

test expr-3.1 {
    expr {1 && 1}
} -ok {1}

test expr-3.2 {
    expr {1 || 1}
} -ok {1}
