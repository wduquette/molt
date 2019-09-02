# Initial set of benchmarks.
pclear

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

benchmark set-1.1 {set var value} {
    set myvar 5
}

benchmark list-1.1 {list of six items} {
    list this that theother foo bar quux
}

pdump
