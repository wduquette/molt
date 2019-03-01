# Test Script: lappend
#
# These tests are based on those in Tcl 8.6 tests/append.test for lappend.
# That file has many tests for traces and other features Molt doesn't have; see
# tests append-7.* and following.

test lappend-1.1 {lappend command} {
    list [lappend x 1 2 abc "long string"] $x
} -ok {{1 2 abc {long string}} {1 2 abc {long string}}}

test lappend-1.2 {lappend command} {
    set x ""
    list [lappend x first] [lappend x second] [lappend x third] $x
} -ok {first {first second} {first second third} {first second third}}

test lappend-1.3 {lappend command} {
    proc foo {} {
  	    global x
	    set x old
	    unset x
	    lappend x new
    }
    foo
    # Should cleanup foo
} -ok {new}

test lappend-1.4 {lappend command} {
    set x {}
    lappend x \{\  abc
} -ok {\{\  abc}

test lappend-1.5 {lappend command} {
    set x {}
    lappend x \{ abc
} -ok {\{ abc}

test lappend-1.6 {lappend command} {
    set x {1 2 3}
    lappend x
} -ok {1 2 3}

test lappend-1.7 {lappend command} {
    set x "a\{"
    lappend x abc
} -ok "a\\\{ abc"

test lappend-1.8 {lappend command} {
    set x "\\\{"
    lappend x abc
} -ok "\\{ abc"

test lappend-1.9 {lappend command} {
    set x " \{"
    lappend x abc
} -error {unmatched open brace in list}

test lappend-1.10 {lappend command} {
    set x "	\{"
    lappend x abc
} -error {unmatched open brace in list}

test lappend-1.11 {lappend command} {
    set x "\{\{\{"
    lappend x abc
} -error {unmatched open brace in list}

test lappend-1.12 {lappend command} {
    set x "x \{\{\{"
    lappend x abc
} -error {unmatched open brace in list}

test lappend-1.13 {lappend command} {
    set x "x\{\{\{"
    lappend x abc
} -ok "x\\\{\\\{\\\{ abc"

test lappend-1.14 {lappend command} {
    set x " "
    lappend x abc
} -ok "abc"

test lappend-1.15 {lappend command} {
    set x "\\ "
    lappend x abc
} -ok "{ } abc"

test lappend-1.16 {lappend command} {
    set x "x "
    lappend x abc
} -ok "x abc"

test lappend-1.17 {lappend command} {
    lappend x
} -ok {}

test lappend-1.18 {lappend command} {
    lappend x {}
} -ok {{}}

if 0 {
    # Need arrays
    test lappend-1.19 {lappend command} {
        lappend x(0)
    } -ok {}

    test lappend-1.20 {lappend command} {
        unset -nocomplain x
        lappend x(0) abc
    } -ok {abc}
}

test lappend-1.21 {lappend command} {
    set x \"
    lappend x
} -error {unmatched open quote in list}

test lappend-1.22 {lappend command} {
    set x \"
    lappend x abc
} -error {unmatched open quote in list}

# TODO: Should be defined in lappend-2.1 -setup
proc check {var size} {
    set l [llength $var]
    if {$l != $size} {
        return "length mismatch: should have been $size, was $l"
    }
    for {set i 0} {$i < $size} {set i [expr $i+1]} {
        set j [lindex $var $i]
        if {$j ne "item $i"} {
            return "element $i should have been \"item $i\", was \"$j\""
        }
    }
    return ok
}

test lappend-2.1 {long lappends} {
    set x ""
    for {set i 0} {$i < 300} {incr i} {
    	lappend x "item $i"
    }
    check $x 300
} -ok ok

# TODO: Need rename; should be done in cleanup.
# rename check {}

test lappend-3.1 {lappend errors} {
    lappend
} -error {wrong # args: should be "lappend varName ?value ...?"}

if 0 {
    # Need arrays
    test lappend-3.2 {lappend errors} {
        set x ""
        lappend x(0) 44
    } -error {can't set "x(0)": variable isn't array}
}
