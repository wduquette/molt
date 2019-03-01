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
    # x contains a list of two elements: "<lbrace><space>" and "abc".
    # It should come out with both the brace and space escaped, but the brace is not escaped.
    # Fix this before going on.
} -ok {\{\  abc}
