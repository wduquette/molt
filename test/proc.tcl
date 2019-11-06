# Test Script: proc command

test proc-1.1 {proc command errors} {
    proc
} -error {wrong # args: should be "proc name args body"}

test proc-1.2 {proc command errors} {
    proc myproc {a {} b} {}
} -error {argument with no name}

test proc-1.3 {proc command errors} {
    proc myproc {a {b 1 extra} c} {}
} -error {too many fields in argument specifier "b 1 extra"}

test proc-2.1 {proc command} -body {
    # Defining a proc returns {}
    proc a {} {}
} -cleanup {
    rename a ""
} -ok {}

test proc-2.2 {proc command} -body {
    # A proc returns the value of evaluating its body
    proc a {} {
        set x 1
    }
    a
} -cleanup {
    rename a ""
} -ok {1}

# Setting a variable in a proc doesn't affect the global scope.
test proc-2.3 {proc command} -body {
    set x 1
    proc a {} {
        set x 2
    }
    set y [a]
    list $x $y
} -cleanup {
    rename a ""
} -ok {1 2}

# Setting a variable in a proc really does set its value in the local scope
test proc-2.4 {proc command} -body {
    set x 1
    set y 2
    proc a {} {
        set x this
        set y that
        list $x $y
    }
    set z [a]
    list $x $y $z
} -cleanup {
    rename a ""
} -ok {1 2 {this that}}

test proc-3.1 {defined proc errors} -body {
    proc myproc {} {}
    myproc a
} -cleanup {
    rename myproc ""
} -error {wrong # args: should be "myproc"}

test proc-3.2 {defined proc errors} -body {
    proc myproc {a {b 1} args} {}
    myproc
} -cleanup {
    rename myproc ""
} -error {wrong # args: should be "myproc a ?b? ?arg ...?"}

test proc-3.3 {defined proc errors} -body {
    # Weird but allowed
    proc myproc {args {b 1} a} {}
    myproc
} -cleanup {
    rename myproc ""
} -error {wrong # args: should be "myproc args ?b? a"}

# Normal argument
test proc-4.1 {defined proc} -body {
    proc myproc {a} {
        list $a $a
    }

    myproc x
} -cleanup {
    rename myproc ""
} -ok {x x}

# Optional argument
test proc-4.2 {defined proc} -body {
    proc myproc {{a A}} {
        list $a
    }

    list [myproc x] [myproc]
} -cleanup {
    rename myproc ""
} -ok {x A}

# Var args
test proc-4.3 {defined proc} -body {
    proc myproc {a args} {
        list $a $args
    }

    list A [myproc 1] B [myproc 1 2] C [myproc 1 2 3]
} -cleanup {
    rename myproc ""
} -ok {A {1 {}} B {1 2} C {1 {2 3}}}

test proc-4.4 {defined proc} -body {
    # Weird but allowed
    proc myproc {args {b 1} a} {list args $args b $b a $a}
    myproc 1 2 3
} -cleanup {
    rename myproc ""
} -ok {args 1 b 2 a 3}
