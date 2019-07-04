# molt bench *filename* ?-csv?

This command executes the benchmark script called *filename* using the Molt benchmark
framework. The framework runs the benchmarks in the script and outputs the results in
microseconds.

```console
$ molt bench benchmarks/basic.tcl
Molt 0.1.0 -- Benchmark

  Micros     Norm -- Benchmark
    0.90     1.00 -- ok-1.1 ok, no arguments
    1.05     1.16 -- ok-1.2 ok, one argument
    1.51     1.67 -- ok-1.2 ok, two arguments
    1.23     1.36 -- ident-1.1 ident, simple argument
    1.46     1.61 -- incr-1.1 incr a
...
$
```

The `Norm` column shows the times relative to the first benchmark in the set.

## CSV Output

 Use the `-csv` option to produce output in CSV format:

 ```console
 $ molt bench benchmarks/basic.tcl -csv
 "benchmark","description","micros","norm"
"ok-1.1","ok, no arguments",0.749,1
"ok-1.2","ok, one argument",1.092,1.457943925233645
"ok-1.2","ok, two arguments",1.509,2.014686248331108
"ident-1.1","ident, simple argument",1.276,1.7036048064085447
"incr-1.1","incr a",1.462,1.9519359145527369
...
```

## Writing Benchmarks

Benchmarks are written using the [**benchmark**](./bench_commands/benchmark.md) or
[**measure**](./bench_commands/measure.md) commands.  See those man pages for examples.
