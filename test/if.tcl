# Test Suite: if command

test if-1.1 {if errors} {
    if
} -error {wrong # args: no expression after "if" argument}

test if-1.2 {if errors} {
    if {true}
} -error {wrong # args: no script following after "true" argument}

test if-1.3 {if errors} {
    if {true} then
} -error {wrong # args: no script following after "then" argument}

test if-1.4 {if errors} {
    if {false} script else
} -error {wrong # args: no script following after "else" argument}

test if-1.5 {if errors} {
    if {false} script elseif
} -error {wrong # args: no expression after "elseif" argument}

# Full syntax, true
test if-2.1 {if command} {
    if {true} then {
        set a "then"
    } else {
        set a "else"
    }
    set a
} -ok {then}

# Minimal syntax, true
test if-2.2 {if command} {
    if {true} {
        set a "then"
    } {
        set a "else"
    }
    set a
} -ok {then}

# No else, true
test if-2.3 {if command} {
    set a "before"
    if {true} {
        set a "then"
    }
    set a
} -ok {then}

# Full syntax, false
test if-2.4 {if command} {
    if {false} then {
        set a "then"
    } else {
        set a "else"
    }
    set a
} -ok {else}

# Minimal syntax, false
test if-2.5 {if command} {
    if {false} {
        set a "then"
    } {
        set a "else"
    }
    set a
} -ok {else}

# No else, false
test if-2.6 {if command} {
    set a "before"
    if {false} {
        set a "then"
    }
    set a
} -ok {before}

# Returns value
test if-3.1 {if command} {
    set a [if {true} { set result "then" }]
    set b [if {false} { set result "then" }]
    set c [if {true} { set result "then" } { set result "else"}]
    set d [if {false} { set result "then" } { set result "else"}]
    list $a $b $c $d
} -ok {then {} then else}

# Handles return properly, true
test if-4.1 {if command} {
    proc doit {x} {
        if {$x} {
            return "then"
        } else {
            return "else"
        }
    }

    list [doit 1] [doit 0]
} -ok {then else}
