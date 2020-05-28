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

# string first
test string-9.1 {string first} {
    string first foo foobarbaz
} -ok 0

test string-9.2 {string first} {
    string first a foobarbaz
} -ok 4

test string-9.3 {string first} {
    string first zoom foobarbaz
} -ok -1

test string-9.4 {string first} {
    string first bar foobarbaz
} -ok 3

test string-9.5 {string first} {
    string first bazz foobarbaz
} -ok -1

test string-9.6 {string first: startIndex} {
    string first bar foobarbaz 3
} -ok 3

test string-9.7 {string first: startIndex} {
    string first bar foobarbaz 5
} -ok -1

test string-9.8 {string first: negative startIndex} {
    string first bar foobarbaz -99
} -ok 3

test string-9.9 {string first: startIndex beyond string end} {
    list \
        [string first z foobarbaz 9] \
        [string first z foobarbaz 10] \
        [string first z foobarbaz 99]
} -ok {-1 -1 -1}

test string-9.10 {string first: non-numerical startIndex} {
    string first a abc NOT_A_NUMBER
} -error {expected integer but got "NOT_A_NUMBER"}

# string trim
test string-10.1 {string trim: empty} {
    string trim {}
} -ok {}

test string-10.2 {string trim: nothing to trim} {
    string trim {hello world}
} -ok {hello world}

test string-10.3 {string trim: whitespace to trim} {
    string trim "    \n\t hello \n\tworld   \t\n   "
} -ok "hello \n\tworld"

# string trimleft
test string-11.1 {string trimleft: empty} {
    string trimleft {}
} -ok {}

test string-11.2 {string trimleft: nothing to trim} {
    string trimleft {hello world}
} -ok {hello world}

test string-11.3 {string trimleft: whitespace to trim} {
    string trimleft "    \n\t hello \n\tworld   \t\n   "
} -ok "hello \n\tworld   \t\n   "

# string trimright
test string-12.1 {string trimright: empty} {
    string trimright {}
} -ok {}

test string-12.2 {string trimright: nothing to trim} {
    string trimright {hello world}
} -ok {hello world}

test string-12.3 {string trimright: whitespace to trim} {
    string trimright "    \n\t hello \n\tworld   \t\n   "
} -ok "    \n\t hello \n\tworld"

# string last
test string-13.1 {string last} {
    string last foo foobarbaz
} -ok 0

test string-13.2 {string last} {
    string last a foobarbaz
} -ok 7

test string-13.3 {string last} {
    string last zoom foobarbaz
} -ok -1

test string-13.4 {string last} {
    string last bar foobarbaz
} -ok 3

test string-13.5 {string last} {
    string last bazz foobarbaz
} -ok -1

test string-13.6 {string last: lastIndex} {
    string last bar foobarbaz 3
} -ok -1

test string-13.7 {string last: lastIndex} {
    string last bar foobarbaz 5
} -ok 3

test string-13.8 {string last: zeno and negative lastIndex} {
    list \
        [string last f foobarbaz 0] \
        [string last f foobarbaz -99]
} -ok {0 -1}

test string-13.9 {string last: lastIndex beyond string end} {
    list \
        [string last z foobarbaz 7] \
        [string last z foobarbaz 9] \
        [string last z foobarbaz 99]
} -ok {-1 8 8}

test string-13.10 {string last: non-numerical lastIndex} {
    string last a abc NOT_A_NUMBER
} -error {expected integer but got "NOT_A_NUMBER"}
