# Test Script: Molt Interpreter

test interp-1.1 {stack level checking} -setup {
    proc bad_recursion {} { bad_recursion }
} -body {
    bad_recursion
} -cleanup {
    rename bad_recursion {}
} -error {too many nested calls to Interp::eval (infinite loop?)}
