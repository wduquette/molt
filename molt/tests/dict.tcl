# Test Script: dict command

# dict create
test dict-1.1 {dict create: odd arguments} {
    dict create a
} -error {wrong # args: should be "dict create ?key value?"}

test dict-1.2 {dict create: no arguments} {
    dict create
} -ok {}

test dict-1.3 {dict create: one key/value pair} {
    dict create a 1
} -ok {a 1}

test dict-1.4 {dict create: multiple key/value pair} {
    set d [dict create a 1 b 2 c 3]

    # TODO: also check that a, b, and c are there and have the right values.
    dict size $d
} -ok {3}

# dict size
test dict-2.1 {dict size: not a dictionary} {
    dict size {a 1 b}
} -error {missing value to go with key}

test dict-2.2 {dict size: empty dict} {
    dict size {}
} -ok {0}

test dict-2.3 {dict size: non-empty dict} {
    dict size {a 1 b 2 c 3}
} -ok {3}
