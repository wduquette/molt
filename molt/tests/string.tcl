# Test Script: string command

test string-2.1 {string cat} {
    list \
        [string cat] \
        [string cat a] \
        [string cat a b]
} -ok {{} a ab}
