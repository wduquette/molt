# source *filename*

Executes the named file as a Molt script, returning the result of the final
command executed in the script.

## TCL Differences

* Standard TCL provides a `-encoding` option, for choosing a specific
  Unicode encoding. Molt assumes that the text read from the file is
  in the UTF-8 encoding, and does nothing special about it.

* Standard TCL reads from the `source`'d file only up to the first ^Z.  
  This allows for the creation of scripted documents: binary files beginning
  with a TCL script.  The script can then open the file and read the rest
  of the data.  Molt does not implement this behavior.
