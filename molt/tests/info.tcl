# Test Script: info command.

test info-1.1 {info errors} {
    info
} -error {wrong # args: should be "info subcommand ?arg ...?"}

# TODO: Really need glob matching.
test info-1.2 {info errors} {
    info nonesuch
} -error {unknown or ambiguous subcommand "nonesuch": must be body, commands, complete, procs, or vars}

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
