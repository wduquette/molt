# Parser Testing
#
# Most of this is done in the Rust tests

# Tests for "empty command at end of script" bug
test parser-1.1 {parser bug fix} {
    set a 1
} -ok 1

# Tests for "not consuming ';' at end of command bug."
test parser-1.2 {parser bug fix} {set a 1; set b 2} -ok 2

# {*} tests

test parser-2.1 {Splat operator at end of input} {list {*}} -ok {*}

test parser-2.2 {Splat operator followed by whitespace} {
    list {*} a b c
} -ok {* a b c}

test parser-2.3 {Splat expands bare word} {
    set x {a b c}
    list - {*}$x -
} -ok {- a b c -}

test parser-2.4 {Splat expands braced word} {
    list - {*}{a b c} -
} -ok {- a b c -}

test parser-2.5 {Splat expands quoted word} {
    list - {*}"a b c" -
} -ok {- a b c -}

test parser-2.6 {Splat expands interpolated script} {
    list - {*}[list a b c] -
} -ok {- a b c -}
