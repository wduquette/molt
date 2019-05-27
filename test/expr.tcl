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

test expr-1.1 {literals} {
    lexpr {1} {-1} {+1}
} -ok {1 -1 1}

test expr-1.2 {literals} {
    lexpr {1.1} {-1.1} {+1.1} {1.1e3} {-1.1e3} {1.1e-3}
} -ok {1.1 -1.1 1.1 1100 -1100 0.0011}

test expr-1.3 {literals} {
    lexpr {1.0} {-1.0} {+1.0}
    # In Tcl, would be "1.0", etc.
    # TODO: Look into floating point compatibility with TCL.
} -ok {1 -1 1}

test expr-1.4 {literals} {
   lexpr true yes on false no off
   # TCL, would return the symbolic constants in this case.
} -ok {1 1 1 0 0 0}

# expr-2.*: arithmetic

test expr-2.1 {arithmetic} {
    lexpr {1 + 2} {3 + 2} {3 + 0}
} -ok {3 5 3}

test expr-2.2 {arithmetic} {
    lexpr {3 - 1} {1 - 3} {3 - 0}
} -ok {2 -2 3}

test expr-2.3 {arithmetic} {
    lexpr {2 * 3} {0 * 2} {2 * 0}
} -ok {6 0 0}

test expr-2.4 {arithmetic} {
    lexpr {4 / 2} {5 / 2} {6 / 2}
} -ok {2 2 3}

test expr-2.5 {arithmetic} {
    expr {2 / 0}
} -error {divide by zero}

test expr-2.6 {arithmetic} {
    lexpr {1 % 4} {3 % 4} {5 % 3}
} -ok {1 3 2}

test expr-2.7 {arithmetic} {
    lexpr {1.1 + 2} {3 + 2.1} {3 + 1.0}
} -ok {3.1 5.1 4}

test expr-2.8 {arithmetic} {
    lexpr {3.1 - 1} {1.1 - 3} {3.1 - 0}
} -ok {2.1 -1.9 3.1}

test expr-2.9 {arithmetic} {
    lexpr {2.5 * 3} {0.0 * 2} {2.0 * 0}
} -ok {7.5 0 0}

test expr-2.10 {arithmetic} {
    lexpr {4.0 / 2} {5 / 2.0} {6.2 / 2}
} -ok {2 2.5 3.1}

test expr-2.11 {arithmetic} {
    expr {2.1 / 0.0}
} -error {divide by zero}

test expr-2.12 {arithmetic} {
    expr {2.2 % 0.0}
} -error {can't use floating-point value as operand of "%"}

test expr-2.13 {sum overflow} {
    # Trouble to double std::i64::MAX
    expr {9223372036854775807 + 9223372036854775807}
} -error {integer overflow}

test expr-2.14 {difference overflow} {
    # Trouble to double std::i64::MAX
    expr {-9223372036854775807 - 9223372036854775807}
} -error {integer overflow}

test expr-2.15 {product overflow} {
    # Trouble to double std::i64::MAX
    expr {2 * 9223372036854775807}
} -error {integer overflow}

test expr-2.16 {negative divisors} {
    expr {1/-2}
} -ok {0}

test expr-2.17 {div/rem consistency} {
    # Per KBK, where a and b are integers and b != 0, / and % must
    # be defined so that:
    #
    # b*(a/b) + (a%b) == a

    set total 0;
    set good 0;

    # Verify for various combinations around zero.
    for {set a -20} {$a <= 20} {incr a} {
        for {set b -20} {$b <= 20} {incr b} {
            if {$b != 0} {
                incr total

                if {$b*($a/$b) + ($a%$b) == $a} {
                    incr good
                }
            }
        }
    }

    expr {$total == $good}
} -ok {1}

# test expr-2.16 {quotient overflow} {
    # Per Google, overflow can occur on signed integer division when for -1/std::i64::MIN.
    # Per Issue #26, you can't currently use std::i64::MIN as an argument at the TCL level.
    # Per Issue #27, integer division for M/N is broken when abs(M) < abs(N) and N < 0.
    # expr {-1 / (-9223372036854775807 - 1)}
# } -error {integer overflow}

# expr-3.*: Logical Operators
proc aflag {flag} {
    global a
    set a "A"
    return $flag
}

proc bflag {flag} {
    global b
    set b "B"
    return $flag
}

test expr-3.1 {logical} {
    lexpr {1 && 1} {1 && 0} {0 && 1} {0 && 0}
} -ok {1 0 0 0}

test expr-3.2 {logical} {
    global a b
    set a ""
    set b ""
    # bflag should not execute.
    set result [expr {[aflag 0] && [bflag 1]}]
    list $result $a $b
} -ok {0 A {}}

test expr-3.3 {logical} {
    lexpr {1 || 1} {1 || 0} {0 || 1} {0 || 0}
} -ok {1 1 1 0}

test expr-3.4 {logical} {
    global a b
    set a ""
    set b ""
    # bflag should not execute.
    set result [expr {[aflag 1] || [bflag 1]}]
    list $result $a $b
} -ok {1 A {}}

test expr-3.5 {logical} {
    lexpr {!1} {!0}
} -ok {0 1}

test expr-3.6 {logical} {
    lexpr {1.1 && 1.1} {1.1 && 0.0} {0.0 && 1.1} {0.0 && 0.0}
} -ok {1 0 0 0}

test expr-3.7 {logical} {
    lexpr {1.1 || 1.1} {1.1 || 0.0} {0.0 || 1.1} {0.0 || 0.0}
} -ok {1 1 1 0}

test expr-3.8 {logical} {
    lexpr {!1.1} {!0.0}
} -ok {0 1}

test expr-3.9 {logical} {
    lexpr {true && true} {true && false} {true || false}
} -ok {1 0 1}

# expr-4.*: Comparisons

test expr-4.1 {comparisons} {
    lexpr {0 == 0} {1 == 0} {0 == 1}
} -ok {1 0 0}

test expr-4.2 {comparisons} {
    lexpr {0 != 0} {1 != 0} {0 != 1}
} -ok {0 1 1}

test expr-4.3 {comparisons} {
    lexpr {0 < 1} {1 < 1} {2 < 1}
} -ok {1 0 0}

test expr-4.4 {comparisons} {
    lexpr {0 <= 1} {1 <= 1} {2 <= 1}
} -ok {1 1 0}

test expr-4.5 {comparisons} {
    lexpr {0 > 1} {1 > 1} {2 > 1}
} -ok {0 0 1}

test expr-4.6 {comparisons} {
    lexpr {0 >= 1} {1 >= 1} {2 >= 1}
} -ok {0 1 1}

test expr-4.7 {comparisons} {
    lexpr {1.1 == 1.1} {1.1 != 1.1} {1.1 < 1.1} {1.1 <= 1.1} {1.1 > 1.1} {1.1 >= 1.1}
} -ok {1 0 0 1 0 1}

# expr-5.*: bitwise operators

test expr-5.1 {bit-wise} {
    lexpr {1 & 2} {2 & 2} {3 & 2}
} -ok {0 2 2}

test expr-5.2 {bit-wise} {
    lexpr {1 | 2} {2 | 2} {3 | 2}
} -ok {3 2 3}

test expr-5.3 {bit-wise} {
    lexpr {~1} {~0}
} -ok {-2 -1}

# expr-6.*: ?: operator

test expr-6.1 {questy} {
    lexpr {1 ? 2 : 3} {0 ? 2 : 3}
} -ok {2 3}

proc a {} {
    global a
    set a 1
    return 1
}

proc b {} {
    global b
    set b 2
    return 2
}

test expr-6.2 {questy} {
    global a b
    set a 0
    set b 0
    set result [expr {0 ? [a] : [b]}]

    # [a] should not have executed.  If it did,
    # $a will be 1.
    list $result $a $b
} -ok {2 0 2}

test expr-6.3 {questy} {
    global a b
    set a 0
    set b 0
    set result [expr {1 ? [a] : [b]}]

    # [b] should not have executed.  If it did,
    # $b will be 2.
    list $result $a $b
} -ok {1 1 0}
