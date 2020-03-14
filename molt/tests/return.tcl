# Test script: return command

# NOTE: The semantics of return are a subset of those of standard TCL.

# Test syntax.  Note: TCL doesn't work this way, but until I implement
# the full return syntax, it doesn't matter.
test return-1.1 {return errors} {
    return foo bar
} -error {wrong # args: should be "return ?value?"}

# return the empty string
test return-2.1 {return command} -setup {
    proc a {} {
        return
    }
} -body {
    a
} -cleanup {
    rename a ""
} -ok {}

# return something else.
test return-2.2 {return command} -setup {
    proc a {} {
        return "howdy"
    }
} -body {
    a
} -cleanup {
    rename a ""
} -ok {howdy}

test return-3.1 {return command: catch} {
    set code [catch {return x} result opts]
    list $code $result $opts
} -ok {2 x {-code 0 -level 1}}
