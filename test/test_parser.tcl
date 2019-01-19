# Parse Testing

test parser-1.1 {
    set a 1
} -ok {1}

test parser-1.2 {set a 1; set a} -ok {1}
