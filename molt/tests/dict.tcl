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

# TODO: test dict-1.4: test multiple entries; but we'll need dict size and dict get first.
