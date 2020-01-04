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
    dict create a 1 b 2 c 3
} -ok {a 1 b 2 c 3}

test dict-1.5 {dict create: duplicated keys} {
    dict create a 1 b 2 b 3 c 4
} -ok {a 1 b 3 c 4}

# dict size
test dict-2.1 {dict size: signature} {
    dict size
} -error {wrong # args: should be "dict size dictionary"}

test dict-2.2 {dict size: not a dictionary} {
    dict size {a 1 b}
} -error {missing value to go with key}

test dict-2.3 {dict size: empty dict} {
    dict size {}
} -ok {0}

test dict-2.4 {dict size: non-empty dict} {
    dict size {a 1 b 2 c 3}
} -ok {3}

# dict get
test dict-3.1 {dict get: signature} {
    dict get
} -error {wrong # args: should be "dict get dictionary ?key ...?"}

test dict-3.2 {dict get: no indices} {
    dict get {a 1}
} -ok {a 1}

test dict-3.3 {dict get: one index} {
    dict get {a 1 b 2 c 3} b
} -ok {2}

test dict-3.4 {dict get: nested indices} {
    dict get {a 1 b {x 2 y 3} c 4} b y
} -ok {3}

test dict-3.5 {dict get: index not found} {
    dict get {a 1 b 2} c
} -error {key "c" not known in dictionary}

test dict-3.6 {dict get: non-dictionary} {
    # Tries to look up "c" in the dictionary "2", which isn't a dictionary
    dict get {a 1 b 2} b c
} -error {missing value to go with key}

test dict-3.7 {dict get: duplicate keys in string rep} {
    dict get {a 1 b 2 b 3 c 4} b
} -ok {3}

# dict exists
test dict-4.1 {dict exists: signature} {
    dict exists
} -error {wrong # args: should be "dict exists dictionary key ?key ...?"}

test dict-4.2 {dict exists: one index, exists} {
    dict exists {a 1 b 2 c 3} b
} -ok {1}

test dict-4.3 {dict exists: one index, no match} {
    dict exists {a 1 b 2 c 3} d
} -ok {0}

test dict-4.4 {dict exists: nested indices, exists} {
    dict exists {a 1 b {x 2 y 3} c 4} b y
} -ok {1}

test dict-4.5 {dict exists: nested indices, no match} {
    dict exists {a 1 b {x 2 y 3} c 4} b z
} -ok {0}

test dict-4.6 {dict exists: non-dictionary} {
    dict exists not-a-dict a
} -ok {0}

test dict-4.7 {dict exists: nested, non-dictionary} {
    dict exists {a 1 b 2} b c
} -ok {0}

# dict set
test dict-5.1 {dict set: signature} {
    dict set
} -error {wrong # args: should be "dict set dictVarName key ?key ...? value"}

test dict-5.2 {dict set: one level} {
    dict set var a 1
    dict set var b 2
    dict set var c 3
    set var
} -ok {a 1 b 2 c 3}

test dict-5.3 {dict set: returns assigned value} {
    dict set var a 1
    dict set var b 2
} -ok {a 1 b 2}

test dict-5.4 {dict set: assign into nested dicts} {
    dict set var a 1
    dict set var b x 2
    dict set var b y 3
} -ok {a 1 b {x 2 y 3}}

test dict-5.5 {dict set: assign into non-dict} {
    dict set var a {x y z}
    dict set var a x 2
} -error {missing value to go with key}

# dict keys
test dict-6.1 {dict keys: signature} {
    dict keys
} -error {wrong # args: should be "dict keys dictionary"}

test dict-6.2 {dict keys: empty} {
    dict keys {}
} -ok {}

test dict-6.3 {dict keys: list of keys} {
    dict keys {a 1 b 2}
} -ok {a b}

# dict values
test dict-7.1 {dict values: signature} {
    dict values
} -error {wrong # args: should be "dict values dictionary"}

test dict-7.2 {dict values: empty} {
    dict values {}
} -ok {}

test dict-7.3 {dict values: list of values} {
    dict values {a 1 b 2}
} -ok {1 2}

# dict remove
test dict-8.1 {dict remove: signature} {
    dict remove
} -error {wrong # args: should be "dict remove dictionary ?key ...?"}

test dict-8.2 {dict remove: empty dictionary, no keys} {
    dict remove {}
} -ok {}

test dict-8.3 {dict remove: empty dictionary, keys} {
    dict remove {} a b c
} -ok {}

test dict-8.4 {dict remove: non-empty dictionary, no keys} {
    dict remove {a 1 b 2 c 3 d 4}
} -ok {a 1 b 2 c 3 d 4}

test dict-8.5 {dict remove: non-empty dictionary, keys} {
    dict remove {a 1 b 2 c 3 d 4} b c e
} -ok {a 1 d 4}
