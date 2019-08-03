# Test Suite: llength command

test llength-1.1 {llength command no args} {
    llength
} -error {wrong # args: should be "llength list"}

test llength-2.1 {llength empty} {
    llength {}
} -ok {0}

test llength-2.2 {llength non-empty} {
    llength {1 2 3}
} -ok {3}
