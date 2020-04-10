# Test script: return command

# NOTE: The semantics of return are a subset of those of standard TCL.

# Test syntax.  Note: TCL doesn't work this way, but until I implement
# the full return syntax, it doesn't matter.
test return-1.1 {return errors} {
    return foo bar
} -error {invalid return option: "foo"}

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

# Test options.

test return-3.1 {return, no options: defaults to ok, 1} {
    set code [catch {return x} result opts]
    list $code $result $opts
} -ok {2 x {-code 0 -level 1}}

test return-3.2 {return ok, 0 is immediate OK} {
    set code [catch {return -code ok  -level 0 x} result opts]
    list $code $result $opts
} -ok {0 x {-code 0 -level 0}}

test return-3.3 {return -code != ok, -level == 0 is immediate} {
    set code [catch {return -code break -level 0 x} result opts]
    list $code $result $opts
} -ok {3 x {-code 3 -level 0}}

test return-3.4 {return -code, -level > 0: is immediate return} {
    set code [catch {return -code break -level 4 x} result opts]
    list $code $result $opts
} -ok {2 x {-code 3 -level 4}}

test return-3.5 {return, -error* ignored if not error.} {
    set code [catch {return -code break -errorcode A -errorinfo B x} result opts]
    list $code $result $opts
} -ok {2 x {-code 3 -level 1}}

test return-3.6 {return, -error* retained for -code error} {
    set code [catch {return -code error -errorcode A -errorinfo B x} result opts]
    list $code $result $opts
} -ok {2 x {-code 1 -errorcode A -errorinfo B -level 1}}

test return-3.7 {return, -error* applied} {
    global errorInfo
    global errorCode
    set code [catch {return -code error -errorcode A -errorinfo B -level 0 x} result opts]
    list $code $result $errorCode [expr {$errorInfo eq [dict get $opts -errorinfo]}]
} -ok {1 x A 1}
