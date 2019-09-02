# Parsed Form

A script is a list of zero or more Commands.

A Command is a list of one or more Words.

A Word is a list of one or more Tokens.

A Token can be:
    *   A Value(Value)
    *   A VarRef(Value)
    *   A CmdRef(Value)

When evaluating a script:

```
For each cmd in commands:
    Build the command's list of values for execution.
    Execute it.

A Token translates to a Value as follows:
    *   If it is a Value, take the Value
    *   If it is a VarRef, take the variable's value.
    *   If it is a CmdRef, evaluate the script in the value

A Word translates to a Value as follows:
    If it is a single token, return the token's value
    If it is multiple tokens, concatenate their values together as a new value.
