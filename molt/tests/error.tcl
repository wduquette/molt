# Test Script: error

test error-1.1 {error error} {
    error
} -error {wrong # args: should be "error message"}

test error-2.1 {error returns error} {
    error "simulated error"
} -error "simulated error"

test error-2.2 {error exits proc} -setup {
    global x
    set x "before"
    proc myproc {} {
        global x
        error "simulated error"
        set x "after"
    }
} -body {
    set code [catch myproc msg]
    list $x $code $msg
} -cleanup {
    rename myproc ""
    global x
    unset x
} -ok {before 1 {simulated error}}

test error-3.1 {error sets return options} {
    set a [catch { error "Message" } result opts]
    list $a $result [dict get $opts -code] [dict get $opts -level] [dict get $opts -errorcode]
} -ok {1 Message 1 0 NONE}
