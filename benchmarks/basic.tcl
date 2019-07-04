# Initial set of benchmarks.

benchmark ok-1.1 {ok, no arguments} {
    ok
}

benchmark ok-1.2 {ok, one argument} {
    ok a
}

benchmark ok-1.3 {ok, two arguments} {
    ok a b
}

benchmark ident-1.1 {ident, simple argument} {
    ident a
}

benchmark incr-1.1 {incr a} {
    incr a
}
