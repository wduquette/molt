# A Tcl implementation of "test", for use running the tests in a
# standard tclsh.  To run:
#
# tclsh tclsh_test.tcl test_script.tcl

proc test {name script option expect} {
    try {
        set result [uplevel #0 $script]

        if {$option eq "-ok" && $result eq $expect} {
            puts "*** test $name passed."
        } else {
            puts "*** test $name FAILED."
            puts "Expected <$expect>"
            puts "Received <$result>"
        }
        return
    } on error result {
        if {$option eq "-error" && $result eq $expect} {
            puts "*** test $name passed."
        } else {
            puts "*** test $name FAILED."
            puts "Expected <$expect>"
            puts "Received <$result>"
        }
        return
    }

    error "Unexpected return code"
}

proc main {argv} {
    source [lindex $argv 0]
}

main $argv
