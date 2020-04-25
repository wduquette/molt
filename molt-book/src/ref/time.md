# time -- Time script execution

**Syntax: time *command* ?*count*?**

Evaluates the given *command* the given number of times, or once if no count is specified,
timing each execution.  The average run time in microseconds is returned as a string,
"*average* microseconds per iteration".

## Example

```tcl
% time { mycommand } 1000
15 microseconds per iteration
%
```
