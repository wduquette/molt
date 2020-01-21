# Test Script: throw

test throw-1.1 {throw signature} {
    throw
} -error {wrong # args: should be "throw type message"}

test throw-2.1 {throw returns error} {
    throw CODE "simulated throw"
} -error "simulated throw"

test throw-2.2 {throw exits proc} -setup {
    global x
    set x "before"
    proc myproc {} {
        global x
        throw CODE "simulated throw"
        set x "after"
    }
} -body {
    set code [catch myproc msg]
    list $x $code $msg
} -cleanup {
    rename myproc ""
    global x
    unset x
} -ok {before 1 {simulated throw}}

test throw-3.1 {throw sets return options} {
    set a [catch { throw CODE "Message" } result opts]
    list $a $result $opts
    # TODO: This will change once the stack trace code is fully implemented.
} -ok {1 Message {-code 1 -level 0 -errorcode CODE -errorinfo Message}}
