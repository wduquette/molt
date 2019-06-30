# Benchmark Library

proc benchmark {name body} {
    puts "$name -- [time $body 1000]"
}
