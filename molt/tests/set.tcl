# Test Script: set command.

test set-1.1 {set sets the variable} {
    set a 1
    list $a
} -ok {1}

test set-1.2 {set sets and returns the variable} {
    set a 1
} -ok {1}

test set-1.3 {set retrieves the variable} {
    set a 1
    set a
} -ok {1}

test set-2.1 {set, no args} {
    set
} -error {wrong # args: should be "set varName ?newValue?"}

test set-2.2 {set, no such variable} {
    set a
} -error {can't read "a": no such variable}
