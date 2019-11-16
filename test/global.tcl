# Test Script: global command.

test global-1.1 {zero args is OK} {
    global
} -ok {}

test global-1.2 {global command} -setup {
    # -body executes in its own scope; so we have to use global to put
    # the variables into the global scope so that [doit] can retrieve them
    # from the global scope.
    proc setx2 {} {
        global x
        set x 2
    }
} -body {
    global x
    set x 1
    setx2
    set x
} -cleanup {
    global x
    unset x
    rename setx2 ""
} -ok {2}

# Can link multiple vars
test global-1.3 {global command} -setup {
    proc setxyz {} {
        global y z
        set x 4
        set y 5
        set z 6
    }
} -body {
    global x y z
    set x 1
    set y 2
    set z 3
    setxyz
    list $x $y $z
} -cleanup {
    global x y z
    unset x y z
    rename setxyz ""
} -ok {1 5 6}

test global-1.4 {global: no such variable} -setup {
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
