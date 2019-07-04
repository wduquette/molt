# ok ?*arg* *arg*...?

**Available in [**molt bench**](../molt_bench.md) scripts only!**

This command takes any number of arguments and returns the empty string.  It is useful when benchmarking code that calls other commands, as (with no arguments) it represents the minimum
amount of computation the Molt interpreter can do.

## Example

For example, Molt's own benchmark suite includes the following as its baseline, as a lower bound on the run-time of evaluating a script:

```tcl
benchmark ok-1.1 {ok, no arguments} {
    ok
}
```
