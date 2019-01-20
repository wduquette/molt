# Language test suite

This folder contains test scripts written using a Tcltest(n)-like `test` command.  The intent is that these tests should pass (with a few exceptions) in both Molt and standard TCL.

To run a test script in Molt:

* `molt test_script.tcl`

To run a test script in standard TCL:

* `tclsh tclsh_test.tcl test_script.tcl`

There is as yet no proper test harness accumulating and summarizing the results of individual tests.
