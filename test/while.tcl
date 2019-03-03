# Test Script: while

test while-1.1 {while error} {
    while
} -error {wrong # args: should be "while test command"}

test while-2.1 {while command} {
    set list {}
    set i 0
    while {$i < 5} {
        lappend list [incr i]
    }
    set list
} -ok {1 2 3 4 5}

test while-2.2 {while executes 0 times if test is always false} {
    set i outside
    while {false} {
        set i inside
    }
} -ok outside

test while-3.1 {while with return} -setup {
    proc myproc {} {
        set i 0
        while {$i < 10} {
            if {$i == 3} {
                return $i
            }
            incr i
        }
    }
} -body {
    myproc
} -cleanup {
    rename myproc ""
} -ok {3}

test while-4.1 {while with break} {
    set i 0
    while {$i < 5} {
        if {$i == 2} {
            break;
        }
        incr i
    }
    set i
} -ok {2}

test while-5.1 {while with continue} {
    set i 0
    set list {}
    while {$i < 10} {
        incr i
        if {$i % 2 == 0} {
            continue
        }
        lappend list $i
    }
    set list
} -ok {1 3 5 7 9}
