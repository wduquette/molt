# benchmark *name* *description* *body* ?*count*?

**Available in [**molt bench**](../molt_bench.md) scripts only!**

Defines a benchmark with the given name and description.  The *body* is a Tcl script; it is executed *count* times via the [**time**](../../ref/time.md) command, and records the average runtime in microseconds.  The count defaults to 1000 iterations.

The *name* should be a symbolic name for easy searching; the *description* should be a
brief human-readable description of the benchmark.

## Example

The following is a simple benchmark of the [**incr**](../../ref/incr.md) command.

```tcl
benchmark incr-1.1 {incr a} {
    incr a
}
```
