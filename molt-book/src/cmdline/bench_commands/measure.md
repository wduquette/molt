# measure *name* *description* *micros*

**Available in [**molt bench**](../molt_bench.md) scripts only!**

This is a low-level command used by the [**benchmark**](./benchmark.md) command
to record measurements.  All recorded measurements will be included in the tool's
output.

Benchmark scripts won't usually need to call this; however, it can
be useful when defining custom benchmarking commands.

## Example

```tcl
measure incr-1.1 "incr a" 1.46
```
