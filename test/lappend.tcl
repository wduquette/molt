# Test Script: lappend

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
