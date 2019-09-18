# What would a byte-compiled script look like?

Registers for temporary values.

*   For each command in the script:
    *   Compute each word in the script, in order.
    *   If the word is a Tokens or a CmdRef, those will need to be byte-compiled as well.
    *   Execute the command, saving its result.
    *   If necessary, return with a non-OK result code
    *   At end, return with the computed result.


The list of byte-codes is an enum; each enum has the tuple data it needs to operate.

One set of registers is just a Vec<Value>; this is what's passed to the command.  As each
argument's value is computed, it's added to the Vec<Value>, building up the command.

What do the standard TCL opcodes look like?
