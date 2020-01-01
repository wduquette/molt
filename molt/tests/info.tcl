# Test Script: info command.

test info-1.1 {info errors} {
    info
} -error {wrong # args: should be "info subcommand ?arg ...?"}

# TODO: Really need glob matching.
test info-1.2 {info errors} {
    info nonesuch
} -error {unknown or ambiguous subcommand "nonesuch": must be args, body, cmdtype, commands, complete, default, exists, procs, or vars}

test info-2.1 {info complete errors} {
    info complete
} -error {wrong # args: should be "info complete command"}

test info-2.2 {info complete errors} {
    info complete foo bar
} -error {wrong # args: should be "info complete command"}

test info-2.3 {info complete command} {
    info complete cmd
} -ok {1}

test info-2.4 {info complete command} {
    info complete "\{cmd"
} -ok {0}

test info-3.1 {info vars command} -setup {
    proc myproc {} {
        info vars
    }
} -body {
    myproc
} -cleanup {
    rename myproc ""
} -ok {}

test info-3.2 {info vars command} -setup {
    proc myproc {a} {
        info vars
    }
} -body {
    myproc a
} -cleanup {
    rename myproc ""
} -ok {a}

test info-3.2 {info vars command} -setup {
    proc myproc {} {
        set v 1
        info vars
    }
} -body {
    myproc
} -cleanup {
    rename myproc ""
} -ok {v}

test info-3.3 {info vars command} -setup {
    proc myproc {} {
        global x
        info vars
    }
} -body {
    myproc
} -cleanup {
    rename myproc ""
} -ok {x}

test info-4.1 {info procs command, added procs} -setup {
    proc thisProc {} {}
    proc thatProc {} {}
} -body {
    set procs [info procs]
    list \
        [expr {"thisProc" in $procs}] \
        [expr {"thatProc" in $procs}] \
        [expr {"set" in $procs}]
} -cleanup {
    rename thisProc ""
    rename thatProc ""
} -ok {1 1 0}

test info-5.1 {info body command, binary command} {
    info body set
} -error {"set" isn't a procedure}

test info-5.2 {info body command, undefined} {
    info body nonesuch
} -error {"nonesuch" isn't a procedure}

test info-5.3 {info body command, defined} -setup {
    proc thisProc {} { puts "Hello, world!" }
} -body {
    info body thisProc
} -cleanup {
    rename thisProc ""
} -ok { puts "Hello, world!" }

test info-6.1 {info args command, binary command} {
    info args set
} -error {"set" isn't a procedure}

test info-6.2 {info args command, undefined} {
    info args nonesuch
} -error {"nonesuch" isn't a procedure}

test info-6.3 {info args command, defined} -setup {
    proc thisProc {a b {c 1}} {}
} -body {
    info args thisProc
} -cleanup {
    rename thisProc ""
} -ok {a b c}

test info-7.1 {info default command, binary command} {
    info default set arg val
} -error {"set" isn't a procedure}

test info-7.2 {info default command, undefined} {
    info default nonesuch arg val
} -error {"nonesuch" isn't a procedure}

test info-7.3 {info default command, undefined arg} -setup {
    proc myproc {arg1 arg2} {}
} -body {
    info default myproc arg3 val
} -cleanup {
    rename myproc ""
} -error {procedure "myproc" doesn't have an argument "arg3"}

test info-7.4 {info default command, no default} -setup {
    proc myproc {arg1 arg2} {}
} -body {
    set flag [info default myproc arg1 val]
    list $flag $val
} -cleanup {
    rename myproc ""
} -ok {0 {}}

test info-7.5 {info default command, default} -setup {
    proc myproc {arg1 {arg2 defval}} {}
} -body {
    set flag [info default myproc arg2 val]
    list $flag $val
} -cleanup {
    rename myproc ""
} -ok {1 defval}

test info-8.1 {info cmdtype command, undefined} {
    info cmdtype nonesuch
} -error {"nonesuch" isn't a command}

test info-8.2 {info cmdtype command} -setup {
    proc myproc {arg1 arg2} {}
} -body {
    list [info cmdtype set] [info cmdtype myproc]
} -cleanup {
    rename myproc ""
} -ok {native proc}

test info-9.1 {info exists command, no such variable} {
    info exists nonesuch
} -ok {0}

test info-9.2 {info exists command, scalar} {
    set a 1
    info exists a
} -ok {1}

test info-9.3 {info exists command, array} {
    set b(1) xyz
    list [info exists b] [info exists b(1)] [info exists b(2)]
} -ok {1 1 0}

test info-9.4 {info exists command, array set} {
    # Creates variable, but it has no items
    array set b {}

    info exists b
} -ok {1}
