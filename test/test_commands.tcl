# Tests for the existing commands.
#
# Ideally we'd be using a subset of Tcl Test, but for now we'll use
# what we have.

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
# puts

# Not tested; can't capture stdout.

#-------------------------------------------------------------------------
# set

test set-1.1 {
    set nonesuch
} -error {can't read "nonesuch": no such variable}

test set-1.2 {
    set
} -error {wrong # args: should be "set varName ?newValue?"}

test set-1.3 {
    set a b c
} -error {wrong # args: should be "set varName ?newValue?"}

test set-2.1 {
    set a 1
} -ok {1}

test set-2.2 {
    set a 2
    set a
} -ok {2}
