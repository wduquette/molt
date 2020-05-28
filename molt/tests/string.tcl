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

test string-3.4 {string compare: -nocase} {
    list \
        [string compare abc ABC] \
        [string compare -nocase abc ABC]
} -ok {1 0}

# string equal
test string-4.1 {string equal: syntax} {
    string equal
} -error {wrong # args: should be "string equal ?-nocase? ?-length length? string1 string2"}

test string-3.2 {string equal: basic} {
    list \
        [string equal a b] \
        [string equal b b] \
        [string equal b a]
} -ok {0 1 0}

test string-3.3 {string equal: -length} {
    list \
        [string equal -length 5 a b] \
        [string equal -length 5 abcdef abcdeg]
} -ok {0 1}

test string-3.4 {string equal: -nocase} {
    list \
        [string equal abc ABC] \
        [string equal -nocase abc ABC]
} -ok {0 1}

# string length
test string-7.1 {string length: syntax} {
    string length
} -error {wrong # args: should be "string length string"}

test string-7.2 {string lengths} {
    list \
        [string length {}] \
        [string length a]  \
        [string length ab] \
        [string length abc]
} -ok {0 1 2 3}

# string tolower
test string-8.1 {string tolower: blank} {
    string tolower {}
} -ok {}

test string-8.2 {string tolower: ASCII} {
    string tolower {ASCII TEXT 0123456789}
} -ok {ascii text 0123456789}

test string-8.3 {string tolower: Unicode} {
    string tolower МАРС
} -ok марс

# string toupper
test string-8.1 {string toupper: blank} {
    string toupper {}
} -ok {}

test string-8.2 {string toupper: ASCII} {
    string toupper {ascii text 0123456789}
} -ok {ASCII TEXT 0123456789}

test string-8.3 {string toupper: Unicode} {
    string toupper венера
} -ok ВЕНЕРА
