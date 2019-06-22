# Test Script: foreach

test foreach-1.1 {foreach argument error} {
    foreach
} -error {wrong # args: should be "foreach varList list body"}

test foreach-1.2 {error in body} {
    foreach x {1 2 3} {
        unset
    }
} -error {wrong # args: should be "unset varName"}

test foreach-2.1 {normal foreach loop} {
    foreach x {1 2 3} {
        append o $x
    }
    set o
} -ok {123}
