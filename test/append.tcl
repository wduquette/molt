# Test Script: append command.

test append-1.1 {append command} {
    unset x
    list [append x 1 2 abc "long string"] $x
} -ok {{12abclong string} {12abclong string}}

test append-1.2 {append command} {
    set x ""
    list [append x first] [append x second] [append x third] $x
} -ok {first firstsecond firstsecondthird firstsecondthird}

test append-1.3 {append command} {
    set x "abcd"
    append x
} -ok abcd

# Need for loop
if 0 {
test append-2.1 {long appends} {
    set x ""
    for {set i 0} {$i < 1000} {set i [expr $i+1]} {
	append x "foobar "
    }
    set y "foobar"
    set y "$y $y $y $y $y $y $y $y $y $y"
    set y "$y $y $y $y $y $y $y $y $y $y"
    set y "$y $y $y $y $y $y $y $y $y $y "
    expr {$x eq $y}
} -ok 1
}

# Need catch
if 0 {
test append-3.1 {append errors} {
    list [catch {append} msg] $msg
} -ok {1 {wrong # args: should be "append varName ?value value ...?"}}

test append-3.2 {append errors} {
    set x ""
    list [catch {append x(0) 44} msg] $msg
} -ok {1 {can't set "x(0)": variable isn't array}}

test append-3.3 {append errors} {
    catch {unset x}
    list [catch {append x} msg] $msg
} -ok {1 {can't read "x": no such variable}}
}
