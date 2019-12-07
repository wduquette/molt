# Test Script: foreach

test foreach-1.1 {foreach argument error} {
    foreach
} -error {wrong # args: should be "foreach varList list body"}

test foreach-1.2 {error in body} {
    foreach x {1 2 3} {
        error "Simulated error"
    }
} -error {Simulated error}

test foreach-2.1 {empty list} {
    set result "0"
    foreach a {} { set result 1}
    set result
} -ok {0}

test foreach-2.2 {loop once per list entry} {
    set result ""
    foreach a {1 2 3} { append result $a}
    set result
} -ok {123}

test foreach-2.3 {stride > 1} {
    set alist ""
    set blist ""
    foreach {a b} {1 2 3} {
        append alist $a
        append blist $b
    }
    list $alist $blist
} -ok {13 2}

test foreach-3.1 {poor man's lassign} {
    foreach {a b c} {1 2 3} {}
    list $a $b $c
} -ok {1 2 3}

test foreach-4.1 {break in loop body} {
    set b "start"
    foreach a {1 2 3} {
        break
        set b "middle"
    }
    list $a $b
} -ok {1 start}

test foreach-4.2 {continue in loop body} {
    set b "start"
    foreach a {1 2 3} {
        continue
        set b "middle"
    }
    list $a $b
} -ok {3 start}
