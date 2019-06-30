# Benchmark Library

# benchmark name description body ?count?
#
# Measures a benchmark, executing the body 1000 times.
proc benchmark {name description body {count 1000}} {
    measure $name $description [lindex [time $body $count] 0]
}
