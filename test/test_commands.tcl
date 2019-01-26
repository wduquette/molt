# Tests for the existing commands.
#
# Ideally we'd be using a subset of Tcl Test, but for now we'll use
# what we have.

#-------------------------------------------------------------------------
# append

test append-1.1 {
    append
} -error {wrong # args: should be "append varName ?value ...?"}

test append-2.1 {
    unset x
    append x
} -ok {}

test append-2.2 {
    unset x
    append x a b c
} -ok {abc}

test append-2.3 {
    unset x
    append x a b c
    append x d e f
} -ok {abcdef}

#-------------------------------------------------------------------------
# exit
#
# Test error cases only, since success would terminate the app.

test exit-1.1 {
    exit foo
} -error {expected integer but got "foo"}

test exit-1.2 {
    exit foo bar
} -error {wrong # args: should be "exit ?returnCode?"}

#-------------------------------------------------------------------------
# global

# Takes any number of arguments, including 0
test global-1.1 {
    global
} -ok {}

# No op at global scope.
test global-1.2 {
    global a b c
} -ok {}

# Links local to global variables
test global-2.1 {
    set x 1
    proc a {} {
        global x
        set x 2
    }
    a
    set x
} -ok {2}

# Can link multiple vars
test global-2.2 {
    set x 1
    set y 2
    set z 3
    proc a {} {
        global y z
        set x 4
        set y 5
        set z 6
    }
    a
    list $x $y $z
} -ok {1 5 6}

#-------------------------------------------------------------------------
# info

test info-1.1 {
    info
} -error {wrong # args: should be "info subcommand ?arg ...?"}

# TODO: really need glob matching or something; as it is, this won't
# pass with tclsh.  Or, I need a way to limit tests to the right
# context, as with tcltest.
test info-1.2 {
    info nonesuch
} -error {unknown or ambiguous subcommand "nonesuch": must be commands, complete, or vars}

test info-2.1 {
    info complete
} -error {wrong # args: should be "info complete command"}

test info-2.2 {
    info complete foo bar
} -error {wrong # args: should be "info complete command"}

test info-2.3 {
    info complete cmd
} -ok {1}

test info-2.4 {
    info complete "\{cmd"
} -ok {0}

#-------------------------------------------------------------------------
# join

test join-1.1 {
    join
} -error {wrong # args: should be "join list ?joinString?"}

test join-2.1 {
    join a
} -ok {a}

test join-2.2 {
    join {a {b c} d}
} -ok {a b c d}

test join-2.3 {
    join {a b} -
} -ok {a-b}

#-------------------------------------------------------------------------
# lindex

test lindex-1.1 {
    lindex
} -error {wrong # args: should be "lindex list ?index ...?"}

test lindex-2.1 {
    lindex {a {b c} d}
} -ok {a {b c} d}

test lindex-2.2 {
    lindex {a {b c} d} 1
} -ok {b c}

test lindex-2.3 {
    lindex {a {b c} d} -1
} -ok {}

test lindex-2.4 {
    lindex {a {b c} d} 3
} -ok {}

test lindex-2.5 {
    lindex {a {b c} d} 1 1
} -ok {c}

test lindex-2.6 {
    lindex {a {b c} d} 1 1 1
} -ok {}

#-------------------------------------------------------------------------
# list
#
# Note: this is intended to cover just the command.  The canonical list
# formatter is tested elsewhere.

test list-1.1 {
    list
} -ok {}

test list-1.2 {
    list a
} -ok {a}

test list-1.3 {
    list a b
} -ok {a b}

test list-1.4 {
    list a {b c} d
} -ok {a {b c} d}

#-------------------------------------------------------------------------
# llength

test llength-1.1 {
    llength
} -error {wrong # args: should be "llength list"}

test llength-2.1 {
    llength {}
} -ok {0}

test llength-2.2 {
    llength {a}
} -ok {1}

test llength-2.3 {
    llength {a b}
} -ok {2}

#-------------------------------------------------------------------------
# proc

# TODO: tests should have oneline descriptions
test proc-1.1 {
    proc
} -error {wrong # args: should be "proc name args body"}

# Defining a proc returns {}
test proc-2.1 {
    proc a {} {}
} -ok {}

# A proc returns the value of evaluating its body
test proc-2.2 {
    proc a {} {
        set x 1
    }
    a
} -ok {1}

# Setting a variable in a proc doesn't affect the global scope.
test proc-2.3 {
    set x 1
    proc a {} {
        set x 2
    }
    set y [a]
    list $x $y
} -ok {1 2}

# Setting a variable in a proc really does set its value in the local scope
test proc-2.4 {
    set x 1
    set y 2
    proc a {} {
        set x this
        set y that
        list $x $y
    }
    set z [a]
    list $x $y $z
} -ok {1 2 {this that}}

#-------------------------------------------------------------------------
# puts

# Not tested; can't capture stdout.

#-------------------------------------------------------------------------
# return
#
# NOTE: The semantics of return are a subset of those of standard TCL.

# Test syntax.  Note: TCL doesn't work this way, but until I implement
# the full return syntax, it doesn't matter.
test return-1.1 {
    return foo bar
} -error {wrong # args: should be "return ?value?"}

# return the empty string
test return-2.1 {
    proc a {} {
        return
    }
    a
} -ok {}

# return something else.
test return-2.2 {
    proc a {} {
        return "howdy"
    }
    a
} -ok {howdy}

#-------------------------------------------------------------------------
# set

test set-1.1 {
    set
} -error {wrong # args: should be "set varName ?newValue?"}

test set-1.2 {
    set a b c
} -error {wrong # args: should be "set varName ?newValue?"}

test set-1.3 {
    set nonesuch
} -error {can't read "nonesuch": no such variable}


test set-2.1 {
    set a 1
} -ok {1}

test set-2.2 {
    set a 2
    set a
} -ok {2}

#-------------------------------------------------------------------------
# set

test unset-1.1 {
    unset
} -error {wrong # args: should be "unset varName"}

test unset-2.1 {
    unset nonesuch
} -ok {}

test unset-2.2 {
    set x 1
    unset x
    set x
} -error {can't read "x": no such variable}
