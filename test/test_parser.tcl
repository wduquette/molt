# Parse Testing

# Tests for "empty command at end of script" bug
test parser-1.1 {
    set a 1
} -ok {1}

# Tests for "empty command at end of script" bug
test parser-1.2 {set a 1; set a} -ok {1}
