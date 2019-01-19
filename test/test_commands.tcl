# Tests for the existing commands.
#
# Ideally we'd be using a subset of Tcl Test, but for now we'll use
# what we have.

# exit
#
# Test error cases only, since success would terminate the app.

test exit-1.1 {
    exit foo
} -error {expected integer but got "foo"}

# puts
#
# Not tested; can't capture stdout.

# set

test set-1.1 {
    set nonesuch
} -error {can't read "nonesuch": no such variable}

test set-1.2 { set a 1 } -ok {1}

test set-1.3 {set a 2; set a} -ok {2}
