# Test Script: array command.

proc match_dicts {expected got} {
    # Length matches?
    if {[llength $expected] != [llength $got]} {
        return 0
    }

    foreach {key value} $expected {
        set e($key) $value
    }

    foreach {key value} $got {
        if {$key ni [array names e]} {
            return 0
        }

        if {$value ne $e($key)} {
            return 0
        }
    }

    return 1
}

proc match_lists {expected got} {
    # Length matches?
    if {[llength $expected] != [llength $got]} {
        return 0
    }

    foreach value $expected {
        if {$value ni $got} {
            return 0
        }
    }

    return 1
}

test array-1.1 {array names, no var} {
    array names
} -error {wrong # args: should be "array names arrayName"}

test array-1.2 {array names, unknown var} {
    array names unknown_variable
} -ok {}

test array-1.3 {array names, scalar var} {
    set scalar 1
    array names scalar
} -ok {}

test array-1.4 {array names, array var} {
    set a(1) one
    set a(2) two
    match_lists {1 2} [array names a]
} -ok {1}

test array-2.1 {array size, no var} {
    array size
} -error {wrong # args: should be "array size arrayName"}

test array-2.2 {array size, unknown var} {
    array size unknown_variable
} -ok {0}

test array-2.3 {array size, scalar var} {
    set scalar 1
    array size scalar
} -ok {0}

test array-2.4 {array size, array var} {
    set a(1) one
    set a(2) two
    array size a
} -ok {2}

test array-3.1 {array exists, no var} {
    array exists
} -error {wrong # args: should be "array exists arrayName"}

test array-3.2 {array exists, unknown var} {
    array exists unknown_variable
} -ok {0}

test array-3.3 {array exists, scalar var} {
    set scalar 1
    array exists scalar
} -ok {0}

test array-3.4 {array exists, array var} {
    set a(1) one
    set a(2) two
    array exists a
} -ok {1}

test array-4.1 {array get, no var} {
    array get
} -error {wrong # args: should be "array get arrayName"}

test array-4.2 {array get, unknown var} {
    array get unknown_variable
} -ok {}

test array-4.3 {array get, scalar var} {
    set scalar 1
    array get scalar
} -ok {}

test array-4.4 {array get, array var} {
    set a(1) one
    set a(2) two
    match_dicts {1 one 2 two} [array get a]
} -ok {1}

test array-5.1 {array unset, no var} {
    array unset
} -error {wrong # args: should be "array unset arrayName ?index?"}

test array-5.2 {array unset, unknown var} {
    array unset unknown_variable
    array exists unknown_variable
} -ok {0}

test array-5.3 {array unset, scalar var} {
    set scalar a
    array unset scalar
    set scalar
} -ok {a}

test array-5.4 {array unset, array var, all elements} {
    set a(1) one
    set a(2) two
    array unset a
    array get a
} -ok {}

test array-5.5 {array unset, array var, one element} {
    set a(1) one
    set a(2) two
    array unset a 1
    array get a
} -ok {2 two}

test array-6.1 {array set, no args} {
    array set
} -error {wrong # args: should be "array set arrayName list"}

test array-6.2 {array set, new var, empty list} {
    array set a {}
    array exists a
} -ok {1}

test array-6.3 {array set, new var, empty list} {
    array set a {x 1 y 2}
    match_dicts {x 1 y 2} [array get a]
} -ok {1}

test array-6.4 {array set, old var, empty list} {
    array set a {x 1 y 2}
    array set a {x 3 z 4}
    match_dicts {x 3 y 2 z 4} [array get a]
} -ok {1}

test array-6.5 {array set on index} {
    catch {
        array set a(1) {a b c d}
    } result1

    catch {
        set a 1
    } result2

    list $result1 $result2
} -ok {{can't set "a(1)": variable isn't array} {can't set "a": variable is array}}

#----------------------------------------------------------------------------
# Cleanup

rename match_dicts ""
rename match_lists ""
