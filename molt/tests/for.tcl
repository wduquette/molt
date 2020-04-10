# Test Script: for

test for-1.1 {for error} {
    for
} -error {wrong # args: should be "for start test next command"}

test for-1.2 {for error in start} {
    for {set} {$i < 5} {incr i} {}
} -error {wrong # args: should be "set varName ?newValue?"}

test for-1.3 {for error in test} {
    for {set i 0} {i < 5} {incr i} {}
} -error {unknown math function "i"}

test for-1.4 {for error in next} {
    for {set i 0} {$i < 5} {incr} {}
} -error {wrong # args: should be "incr varName ?increment?"}

test for-1.5 {for error in body} {
    for {set i 0} {$i < 5} {incr i} {
        error "Simulated error"
    }
} -error {Simulated error}

test for-2.1 {for loop with break} {
    set a {}
    for {set i 1} {$i < 6} {incr i} {
        if {$i == 4} {
            break
        }
        lappend a $i
    }
    set a
} -ok {1 2 3}

test for-2.2 {for loop with break, nested loop} {
    set a {}
    for {set i 1} {$i < 4} {incr i} {
        for {set j 1} {$j < 4} {incr j} {
            if {$j == 2} {
                break
            }
            lappend a "$i,$j"
        }
    }
    set a
} -ok {1,1 2,1 3,1}

test for-2.3 {for loop with continue} {
    set a {}
    for {set i 1} {$i < 6} {incr i} {
        if {$i == 4} {
            continue
        }
        lappend a $i
    }
    set a
} -ok {1 2 3 5}

test for-2.4 {for loop with continue, nested loop} {
    set a {}
    for {set i 1} {$i < 4} {incr i} {
        for {set j 1} {$j < 4} {incr j} {
            if {$j == 2} {
                continue
            }
            lappend a $i,$j
        }
    }
    set a
} -ok {1,1 1,3 2,1 2,3 3,1 3,3}

test for-2.5 {for loop result} {
    for {set i 0} {$i < 5} {incr i} {
        set a $i
    }
} -ok {}

test for-3.1 {break in for start} {
    catch {
        for {break} {1} {} {}
    }
} -ok {3}

test for-3.2 {break in for test} {
    for {} {[break]} {} {}
} -error {invoked "break" outside of a loop}

test for-3.3 {break in for next} {
    for {} {1} {break} {}
} -ok {}

test for-3.4 {continue in for start} {
    catch {
        for {continue} {1} {} {}
    }
} -ok {4}

test for-3.5 {continue in for test} {
    for {} {[continue]} {} {}
} -error {invoked "continue" outside of a loop}

test for-3.6 {continue in for next} {
    for {} {1} {continue} {}
} -error {invoked "continue" outside of a loop}
