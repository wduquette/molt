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
# assert_eq

test assert_eq-1.1 {
    assert_eq
} -error {wrong # args: should be "assert_eq received expected"}

test assert_eq-2.1 {
    assert_eq a a
} -ok {}

test assert_eq-2.2 {
    assert_eq a b
} -error {assertion failed: received "a", expected "b".}

#-------------------------------------------------------------------------
# break
#
# This will be tested in the context of each kind of loop.
#
# TODO: test return value with "catch" when that's available.

test break-1.1 {
    break
} -error {invoked "break" outside of a loop}

#-------------------------------------------------------------------------
# continue
#
# This will be tested in the context of each kind of loop.
#
# TODO: test return value with "catch" when that's available.

test continue-1.1 {
    continue
} -error {invoked "continue" outside of a loop}

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
# foreach

test foreach-1.1 {
    foreach
} -error {wrong # args: should be "foreach varList list body"}

# Doesn't execute if there's no list data.
test foreach-2.1 {
    set result "0"
    foreach a {} { set result 1}
    set result
} -ok {0}

# Executes once per list entry
test foreach-2.2 {
    set result ""
    foreach a {1 2 3} { append result $a}
    set result
} -ok {123}

# Stride > 1
test foreach-2.3 {
    set alist ""
    set blist ""
    foreach {a b} {1 2 3} {
        append alist $a
        append blist $b
    }
    list $alist $blist
} -ok {13 2}

# Poor man's lassign
test foreach-3.1 {
    foreach {a b c} {1 2 3} {}
    list $a $b $c
} -ok {1 2 3}

# Break
test foreach-4.1 {
    set b "start"
    foreach a {1 2 3} {
        break
        set b "middle"
    }
    list $a $b
} -ok {1 start}

# Continue
test foreach-4.2 {
    set b "start"
    foreach a {1 2 3} {
        continue
        set b "middle"
    }
    list $a $b
} -ok {3 start}

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
# if
#
# TODO: All of these will need to be updated once we have expression
# parsing.

test if-1.1 {
    if
} -error {wrong # args: no expression after "if" argument}

test if-1.2 {
    if {set x 1}
} -error {wrong # args: no script following after "set x 1" argument}

test if-1.3 {
    if {set x 1} then
} -error {wrong # args: no script following after "then" argument}

test if-1.4 {
    if {set x 0} script else
} -error {wrong # args: no script following after "else" argument}

test if-1.5 {
    if {set x 0} script elseif
} -error {wrong # args: no expression after "elseif" argument}

# Full syntax, true
test if-2.1 {
    # TODO: Fix when we have expressions
    if {set x 1} then {
        set a "then"
    } else {
        set a "else"
    }
    set a
} -ok {then}

# Minimal syntax, true
test if-2.2 {
    # TODO: Fix when we have expressions
    if {set x 1} {
        set a "then"
    } {
        set a "else"
    }
    set a
} -ok {then}

# No else, true
test if-2.3 {
    # TODO: Fix when we have expressions
    set a "before"
    if {set x 1} {
        set a "then"
    }
    set a
} -ok {then}

# Full syntax, false
test if-2.4 {
    # TODO: Fix when we have expressions
    if {set x 0} then {
        set a "then"
    } else {
        set a "else"
    }
    set a
} -ok {else}

# Minimal syntax, false
test if-2.5 {
    # TODO: Fix when we have expressions
    if {set x 0} {
        set a "then"
    } {
        set a "else"
    }
    set a
} -ok {else}

# No else, false
test if-2.6 {
    # TODO: Fix when we have expressions
    set a "before"
    if {set x 0} {
        set a "then"
    }
    set a
} -ok {before}

# Returns value
test if-3.1 {
    # TODO: Fix when we have expressions
    set a [if {set x 1} { set result "then" }]
    set b [if {set x 0} { set result "then" }]
    set c [if {set x 1} { set result "then" } { set result "else"}]
    set d [if {set x 0} { set result "then" } { set result "else"}]
    list $a $b $c $d
} -ok {then {} then else}

# Handles return properly, true
test if-4.1 {
    proc doit {x} {
        if {set x} {
            return "then"
        } else {
            return "else"
        }
    }

    list [doit 1] [doit 0]
} -ok {then else}

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

test info-3.1 {
    proc myproc {} {
        info vars
    }
    myproc
} -ok {}

test info-3.2 {
    proc myproc {a} {
        info vars
    }
    myproc a
} -ok {a}

test info-3.2 {
    proc myproc {} {
        set v 1
        info vars
    }
    myproc
} -ok {v}

test info-3.3 {
    proc myproc {} {
        global x
        info vars
    }
    myproc
} -ok {x}

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

test list-1.5 {
    list a {} c
} -ok {a {} c}

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

test proc-1.2 {
    proc myproc {a {} b} {}
} -error {argument with no name}

test proc-1.3 {
    proc myproc {a {b 1 extra} c} {}
} -error {too many fields in argument specifier "b 1 extra"}

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

test proc-3.1 {
    proc myproc {} {}
    myproc a
} -error {wrong # args: should be "myproc"}

test proc-3.2 {
    proc myproc {a {b 1} args} {}
    myproc
} -error {wrong # args: should be "myproc a ?b? ?arg ...?"}

test proc-3.3 {
    # Weird but allowed
    proc myproc {args {b 1} a} {}
    myproc
} -error {wrong # args: should be "myproc args ?b? a"}

# Normal argument
test proc-4.1 {
    proc myproc {a} {
        list $a $a
    }

    myproc x
} -ok {x x}

# Optional argument
test proc-4.2 {
    proc myproc {{a A}} {
        list $a
    }

    list [myproc x] [myproc]
} -ok {x A}

# Var args
test proc-4.3 {
    proc myproc {a args} {
        list $a $args
    }

    list A [myproc 1] B [myproc 1 2] C [myproc 1 2 3]
} -ok {A {1 {}} B {1 2} C {1 {2 3}}}

test proc-4.4 {
    # Weird but allowed
    proc myproc {args {b 1} a} {list args $args b $b a $a}
    myproc 1 2 3
} -ok {args 1 b 2 a 3}

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
