# Test Script: assert_eq

test assert_eq-1.1 {assert_eq command} {
    assert_eq a a
} -ok {}

test assert_eq-1.2 {assert_eq command} {
    assert_eq a b
} -error {assertion failed: received "a", expected "b".}

test assert_eq-2.1 {assert_eq errors} {
    assert_eq
} -error {wrong # args: should be "assert_eq received expected"}
