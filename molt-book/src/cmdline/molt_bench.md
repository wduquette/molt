# molt bench *filename* ?-csv?

This command executes the benchmark script called *filename* using the Molt benchmark
framework. The framework runs the benchmarks in the script and outputs the results in
nanoseconds.

**NOTE:** The benchmark tool is experimental, subject to change, and primarily intended
as aid for Molt optimization.

The output looks like this.

```console
$ molt bench benchmarks/basic.tcl
Molt 0.2.0 -- Benchmark

   Nanos     Norm -- Benchmark
    3344     1.00 -- ok-1.1 ok, no arguments
    4110     1.23 -- ok-1.2 ok, one argument
    4442     1.33 -- ok-1.3 ok, two arguments
    4005     1.20 -- ident-1.1 ident, simple argument
    7175     2.15 -- incr-1.1 incr a
    6648     1.99 -- set-1.1 set var value
    7926     2.37 -- list-1.1 list of six items
...
$
```

The `Norm` column shows the times relative to the first benchmark in the set.

## CSV Output

 Use the `-csv` option to produce output in CSV format:

 ```console
 $ molt bench benchmarks/basic.tcl -csv
 "benchmark","description","nanos","norm"
 "ok-1.1","ok, no arguments",3313,1
 "ok-1.2","ok, one argument",4027,1.2155146392997283
 "ok-1.3","ok, two arguments",4439,1.3398732266827649
 "ident-1.1","ident, simple argument",4026,1.2152127980682161
 "incr-1.1","incr a",7325,2.210987020827045
 "set-1.1","set var value",6499,1.9616661635979475
 "list-1.1","list of six items",7848,2.3688499849079383
...
```

## Writing Benchmarks

Benchmarks are written using the [**benchmark**](./bench_commands/benchmark.md) or
[**measure**](./bench_commands/measure.md) commands.  See those man pages for examples.
