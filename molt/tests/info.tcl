# Test Script: info command.


test info-1.1 {info errors} {
    info
} -error {wrong # args: should be "info subcommand ?arg ...?"}

# TODO: really need glob matching or something; as it is, this won't
# pass with tclsh.  Or, I need a way to limit tests to the right
# context, as with tcltest.
test info-1.2 {info errors} {
    info nonesuch
} -error {unknown or ambiguous subcommand "nonesuch": must be commands, complete, or vars}

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
