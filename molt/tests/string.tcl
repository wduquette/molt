# Test Script: string command

# string cat
test string-2.1 {string cat} {
    list \
        [string cat] \
        [string cat a] \
        [string cat a b]
} -ok {{} a ab}

# string compare
test string-3.1 {string compare: syntax} {
    string compare
} -error {wrong # args: should be "string compare ?-nocase? ?-length length? string1 string2"}

test string-3.2 {string compare: basic} {
    list \
        [string compare a b] \
        [string compare b b] \
        [string compare b a]
} -ok {-1 0 1}

test string-3.3 {string compare: -length} {
    list \
        [string compare -length 5 a b] \
        [string compare -length 5 abcdef abcdeg]
} -ok {-1 0}

test string-3.3 {string compare: -nocase} {
    list \
        [string compare abc ABC] \
        [string compare -nocase abc ABC]
} -ok {1 0}
