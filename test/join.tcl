# Test Script: join command.

test join-1.1 {one element, no join string} {
    join a
} -ok {a}

test join-1.2 {one element, join string} {
    join a ,
} -ok {a}

test join-1.3 {multiple elements, no join string} {
    join [list a b c]
} -ok {a b c}

test join-1.4 {multiple elements, join string} {
    join [list a b c] ,
} -ok {a,b,c}
