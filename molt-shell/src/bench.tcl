# Benchmark Library

proc benchmark {name description body} {
    puts "$name -- [time $body 1000]"
}
