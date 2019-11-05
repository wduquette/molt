# Test Script: global command.

test global-1.1 {global} -setup {
    proc doit {} {
        global a b
        return [list $a $b]
    }
} -body {
    # -body executes in its own scope; so we have to use global to put
    # the variables into the global scope so that [doit] can retrieve them
    # from the global scope.
    global a b
    set a 1
    set b 2
    doit
} -cleanup {
    global a b
    unset a b
    rename doit ""
} -ok {1 2}

test global-1.2 {global: no such variable} -setup {
    # Make sure a isn't defined.
    global a
    unset a
    proc doit {} {
        global a
        return $a
    }
} -body {
    doit
} -cleanup {
    rename doit ""
} -error {can't read "a": no such variable}
