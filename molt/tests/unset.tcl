# Test Script: unset command.

test unset-1.1 {unset, various no-ops} {
    unset
    unset a
    unset -nocomplain a
    unset -ncomplain -- a
} -ok {}

test unset-1.2 {unset, one variable} {
    set a 1
    unset a
    set a
} -error {can't read "a": no such variable}

test unset-1.3 {unset, two variables} {
    set a 1
    set b 2
    unset a b
    catch {set a} result1
    catch {set b} result2
    list $result1 $result2
} -ok {{can't read "a": no such variable} {can't read "b": no such variable}}

test unset-1.4 {unset, array variable} {
    set a(1) one
    set a(2) two
    unset a
    set a
} -error {can't read "a": no such variable}

test unset-1.5 {unset, array element} {
    set a(1) one
    set a(2) two
    unset a(2)
    set a(2)
} -error {can't read "a(2)": no such element in array}
