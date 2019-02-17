# Molt Development Journal

### 2019-02-17
*   Expression Parsing.
    *   Tcl 7.6 parses integers using `strtoul` with a base of "0", which
        means that it will accept "0x..." as hex, "0..." as octal, and
        anything else as decimal.  
    *   Further, `strtoul` parses the integer from the beginning of a string,
        and returns a pointer to the next character.
    *   There is no equivalent to `strtoul` in Rust.
    *   I don't want or need the octal conversion anyway.
    *   Thing to do: write a function that takes a CharPtr and returns
        Option<String>.  The CharPtr will point to the next character, and the
        String will contain the text of the integer.  Then, use `molt_get_int()`
        to convert it to a MoltInt.
        *   Then, both functions can eventually be extended to support hex.
    *   Similar for MoltFloat.
    *   Added util::read_int() and util::read_float(), with tests.
    *   Added CharPtr::skip_over().

### 2019-02-16
*   Expression Parsing.
*   Starting with Tcl 7.6 parser, per Don Porter.
*   The expression parser often needs to look ahead, i.e., try to parse an integer.
    If it succeeds, the ExprContext needs to get updated to point to the next thing.
    We can do this by cloning the iterator.
    *   info.chars points at the next character in the input.
    *   let mut p = info.chars.clone() gives us the ability to work farther along.
    *   info.chars = p updates the pointer to the next thing.
    *   Built this into CharPtr.
*   Note: you can't easily compare two iterators for equality the way you can
    compare two `char*`'s.  Better approach: ask parsing routine to return
    `Result<Option<_>,ResultCode>`.  Keep the cloned iterator if Some, and
    not if None.
    *   Could define CharPtr to keep the input `&str` and use `enumerate()` to
        get the index of each character.  Then I could compare these for
        equality.
    *   Tried this; it appears to work.  I'd rather avoid it, though.

### 2019-02-03
*   Expression Parsing
    *   Spent some time yesterday looking at Nom and Pest.
        *   Nom is a parser combinator library.
        *   Pest is a PEG parser generator.
    *   Observations:
        *   Pest has better intro documentation (though it's not perfect).
        *   Pest uses a grammar definition file, Nom does not.
        *   Pest has built-in support for operator precedence and a partial
            example of its use, and Nom does not.
        *   It's much easier to integrate existing code with Nom code:
            Nom parsers are built up of smaller parsers, which can be
            hand-coded if need be.
            *   E.g., I can handle interpolated scripts using
                a suitably-wrapped call to `Interp::eval()`.
        *   Big difficulty with Pest: it wants to do the whole job.  I'd
            need to include rules for command interpolation strings.
        *   Difficulty with Nom: parsing for operator precedence.
            *   Could use an example.
            *   I know people at JPL who have used Nom, who might be able
                to help.
        *   Nom seems like the better choice going forward, if I can
            figure out how to use it.
    *   Need to read up on parsing algebraic expressions.
    *   Two approaches:
        *   Evaluate as I go.  This can work if the structure of the parser
            matches the operator precedence.
        *   "Compile" and evaluate the syntax tree.
    *   The latter lends itself to byte compilation in a way that
        the former doesn't, and is probably no more work to write, once
        I've defined the syntax tree data structure.
        *   So that's probably the next task.
*   Sent email to Don Porter and Andreas Kupries asking for advice.    

### 2019-02-02 (Saturday)
*   Added "if" command.
    *   Since we have no expression parser yet, the conditions
        are assumed to be scripts returning a boolean value.
*   Added "foreach" command.
    *   Only iterates over a single list.
*   Added "break" and "continue" commands.

### 2019-01-27 (Sunday)
*   molt language
    *   Added an `assert_eq` command, for use in molt-book examples, a la the
        Rust documentation.
*   molt-book
    *   Added docs for `assert_eq` and `test`, with examples.
    *   Added example for `append`, using `assert_eq`.
*   Added `molt_ok!` and `molt_err!` macros.
    *   Which are now used throughout.

### 2019-01-26 (Saturday)
*   First, I need to replace the Interp's variable map with a VarStack.
    *   Done.  `Interp::{set,get,unset}` all work as expected.
*   Next, `proc` needs to push and pop stacks.
    *   Done, and tested.  Variables in procs really do have local scope.
*   Implemented `return`.
    *   Subset of full TCL behavior; no way to use return for anything
        but a standard return.
    *   Issue: when to convert a Return result to a normal result?
    *   In TCL the `eval` command clearly does this and proc bodies
        do this, both of which are appropriate.
        *   Procs, because that's what "return" is for.
        *   `eval`, because that's what's used for interpreting interactive
            commands, and that's how it responds.
    *   But control structures like "if" and "while" will need to let
        "return" pass through unchanged.
    *   In TCL, you'd use `catch` instead of `eval`; I suppose I'll need
        an Interp method for that.
    *   Meanwhile, `proc` bodies can use standard `eval` semantics and
        not worry about handling `return` explicitly.
    *   Which is why Tcl_Eval() converts break and continue results to
        errors!  Got it!
*   Implemented `global`, with tests.
*   Argument processing.
    *   Got basic processing down, including optional and var args.
        *   Found bug in list_to_string(); empty arguments weren't braced.
            Fixed.
    *   Added proper "wrong # args" output
*   Proc argument validation
    *   Playing with TCL, found the following behavior:
        *   `proc` rejects arguments that aren't lists of 1 or 2 elements.
        *   `proc` does **not** check whether required/optional/args are
            in the correct order.
            *   In `proc {{a 1} b} {...}` the default for `a` can never
                be used, so it's irrelevant; but it isn't flagged as an error.
        *   The `args` argument is treated as a normal argument if it's
            not in the last place.
        *   Added `proc` arg spec length checking:
            *   "argument with no name"
            *   "too many fields in argument specifier "b 1 2""
        *   Fixed CommandProc's argument processing for "args".

### 2019-01-25 (Friday)
*   Added tests for the VarStack struct.
    *   Found only one bug, in `upvar()`.  Fixed.
*   Now I have the infrastructure I need for proc arguments and local
    variables.

### 2019-01-24 (Thursday)
*   Added the VarStack struct, for managing an interpreter's variables.
    *   Maintains all variable scopes for the interpreter.
    *   Supports upvar/global.

### 2019-01-23 (Wednesday)
*   Added a stub for `proc`.  At present it just executes the body in
    the global context, ignoring the arguments completely.
*   Discovered that I didn't completely fix the "empty command at end
    of script bug"; or, at least, there's another bug.
    *   The following script returns 1; it should return 2.
        *   `set a 1; set b 2`

### 2019-01-21 (Monday)
*   Added `list`, `llength`, `lindex` (with tests)
*   Implementing `proc`
    *   `proc` presents these difficulties
        *   Argument processing
            *   Handling optional arguments and the args argument
                *   This is tedious, but not tricky.
                *   Need not be done for initial implementation.
        *   Variable management
            *   Variables are local by default (including arguments)
            *   Global variables are available only when declared.
            *   We will support the `global` command, but not the
                `variable` command for now.
                *   Variable only really makes sense with namespaces.
        *   Return handling
            *   A return from the middle of the body needs to be handled
                correctly.
                *   I think I have all of the ground work done for this.
    *   Handling local variables
        *   First, store variables in a VarMap: a struct wrapping a
            `HashMap<String, RefCell<String>>`
            *   Question: what happens if you're looping over a list and
                and modify the list contents in TCL?  I'm pretty sure the
                loop is effectively over a copy, because of copy-on-write
                semantics.
        *   Next, the Interp has a stack of VarMaps.  The bottom entry
            reflects the global scope (`upvar #0`!).
        *   Next, each proc pushes a VarMap on the stack, and initializes
            it with its arguments.
        *   `set` and variable interpolation refer to the top of the stack.
        *   The `global` command copies the RefCell for a global variable
            into the top VarMap (and creates it in the global namespace as
            well, if need be)
        *   When the proc returns, it pops its varmap off of the stack.
            *   Could/should we do this in a Drop handler?

### 2019-01-20 (Sunday)
*   Found a write-up on proper Tcl list syntax in the Tcl code base,
    courtesy of wiki.tcl-lang.org.  See docs/list_format.txt.
    *   Two challenges: parsing lists into vectors, and formatting vectors
        as valid lists.
    *   Parsing lists into vectors:
        *   This is almost but not quite the same as parsing a command
            without evaluation (as you'd do for `info complete`).
        *   List of differences:
            *   The set of whitespace characters for lists consists
                of " ", \t, \n, \v, \f, \r.
            *   It includes \n which is a command terminator
            *   It does *not* include other unicode whitespace characters.
                *   It appears that these ARE valid whitespace during
                    command parsing.
            *   backslash-newline is NOT a whitespace character
        *   Backslash processing is done as for command parsing.
        *   TODO: When a backslash is the final character in a string,
            it is replaced by itself. (!)
            *   Should be true for command parsing as well.
        *   Braced-words are parsed as for commands, EXCEPT THAT
            backslash-newline is passed along unchanged rather than
            being converted to a space.
        *   Quoted words are processed without looking for interpolated
            variables or commands; in particular, single "[" characters
            are not an error.  The words ends with the first unescaped
            double-quote.  THEN, backslash substitution is done on the
            word AFTER the full word is known.
        *   Bare words are processed without looking for variables or
            commands, reading up to the next list whitespace, and then
            backslash substitution is done on the result.
        *   For both quoted and bare words, the rule seems to be that
            escapes are preserved without processing (so backslash-space)
            does not break a word; then backslash-substitution is done
            on the word after.
    *   Formatting vectors as lists:
        *   This is simply joining the words with whitespace—except that
            the words may need to be quoted.
        *   Details:
            *   TODO
*   Conclusions:
    *   This is sufficiently different from command parsing as to require
        distinct code (though some lower-level routines can be shared,
        e.g., the Context struct).  It doesn't make sense to confuse the
        command parsing routines with a bunch of if-tests.
    *   This is NOT `info complete` checking, which does need to be
        part of the command parser.
*   Added Interp::complete() and "info complete".
    *   As part of doing "info complete" I added a subcommand mechanism
        and stubs for "info commands" and "info vars".
*   Added list parsing and formatting: `crate::list::get_list()` and
    `crate::list::list_to_string()`.

### 2019-01-19 (Saturday)
*   Added `set` command
    *   Because it makes it easier to test the parser.
*   Spent some time on documentation and tests.
    *   Moved the basic utility functions to lib.rs, since the user will
        want to use them as, e.g., `molt::check_args()`.
*   Added command interpolation.
*   Added variable interpolation.
*   Added backslash substitution.
*   Changed name to "molt".
*   Added "test" command, kind of a baby TclTest command.
*   Began adding tests for the current command set.
    *   test/test_commands.tcl
    *   Found a parsing bug
    *   Added test/test_parser.tcl to verify the bug and the fix.
    *   Fixed it.
    *   Completed the tests for `exit`, `puts`, `set`
        *   Couldn't really test `puts`, as I don't capture stdout.
*   Ultimately I'm going to want a fancier test harness.  I'm thinking of
    a set of subcommands:
    *   molt shell
    *   molt run
    *   molt test


### 2019-01-18 (Friday)
*   Extended `Interp::eval` with the basic parser skeleton.
    *   Handles comments
    *   Handles quoted and braced strings.
    *   Does not yet handle backslash escapes, or command and variable
        interpolation.

### 2019-01-16 (Wednesday)
*   Added `Context` struct as a parsing aid.

### 2019-01-14 (Monday)
*   Result handling:
    *   `Result<>` and the `?` syntax is the way Rust provides non-local
        returns back through the call stack.
    *   That's the behavior we want for all of the result codes except
        TCL_OK.
    *   Thus, TCL_OK really is `Ok(_)`, while TCL_RETURN, TCL_BREAK, TCL_CANCEL,
        and TCL_ERROR are all flavors of `Err(_)`.
    *   The T in `Ok(T)` can reasonably change from one function to another,
        i.e., in `molt::get_int()` T can be `i32`, which in `molt::eval()` it
        can be a String.  But the E in `Err(E)` really needs to be the same,
        so that it can propagate.
    *   So what I want is `type InterpResult = Result<String,ResultCode>` where
        `ResultCode` is `enum {Error(String), Return(String), Break, Continue}`.
    *   I can mitigate the nuisance of using these codes in client code by
        defining some helper functions, e.g., `fn error(String) -> InterpResult`.

### 2019-01-13 (Sunday)
*   Beginning to try to replicate the Tcl 7.6 Tcl_Eval in Rust.
    *   Was able to follow Tcl_Eval for the most part.
        *   Although it uses some gotos.
    *   But Tcl_Eval calls TclParseWords, which implements a finite
        state machine using gotos in a way that simply isn't easily
        translatable into Rust.  Humph.
    *   And, of course, there's lots of low-level memory management
        foofaraw that doesn't apply to what I'm doing.
    *   And Ousterhout et al were a little casual about their
        indentation.
    *   I might actually be better off trying to implement the
        Octalogue directly (https://wiki.tcl-lang.org/page/Dodekalogue).
*   What have I learned?
    *   Tcl 7.6 isn't a great example (though it will help)
        *   Still highly optimized.
        *   Logic is obscured by C memory handling
            *   Rust vectors and iterators should help a lot.
    *   Requirements:
        *   Ability to parse without evaluating (e.g., "info complete")
        *   Need to track stack depth, and cut it short before the Rust
            app blows up.
            *   Therefore, using "?" syntax isn't desirable in
                molt::eval, as I can't decrement the stack depth.
    *   Result and result code handling
        *   Tcl's interp result and return code scheme isn't a great match
            for Rust's `Result<>` type.  What you want to do is add
            Break, Continue, and Return(val).
        *   It's probably easier to add a "result" String to interp, to be
            set for error messages and other result values, and add
            a ResultCode enum for use with Ok.
        *   Begin as you mean to go on: the "result" object probably should
            wrap a string rather than be a string, so I can implement
            something like Tcl_Obj in the long run.
*   Tried this; don't like it.
    *   Using `Result<>` is handy for check_args, etc., but otherwise
        a nuisance.
        *   Better to use a custom return enum, and write a macro
            like try!().
        *   Did that. (Except for try!)
*   Parsing context
    *   Simple struct.  Contains a peekable iterator, and parsing
        flags.  eval() creates it, passes it to eval_script() (new), which
        parses and evaluates commands.
        *   Flags: parse only, looking for square bracket or not,
            maybe stack depth.

## 2019-01-09 (Wednesday)
*   Extended the Interp::eval() method to actually parse a script, execute
    each command, and return the result of the last command.
    *   It throws the proper error if a command isn't remembered.
*   The shell now calls Interp::eval() when it ought to.
*   Next steps:
    *   Add set and unset methods, and variable storage.
    *   Reorganize the code.
    *   Add string quoting and interpolation to the parser.
    *   Add list processing.
*   Basic principle for now: naive is fine.  Make it naive, and write
    a test suite.  Then I can make it smarter.

## 2019-01-08 (Tuesday)
*   Revised to use command storage and traits as described in yesterday's
    journal entry.
*   Added a rustyline-based shell, but it doesn't actually call the interpreter
    yet.

## 2019-01-07 (Monday)

*   Insight: Command Storage
    *   The commands map is a map from String to Rc<CommandEnum>
    *   A command can be a built-in command function, or a proc.
        CommandEnum can handle both cases.
    *   Commands are *always* looked up by name at run time.
        *   Because the name can be changed, and procs can be
            redefined.
    *   Basic Principle: Tcl is evaluated *as though* the scripts and
        proc bodies were saved as Strings, not byte-compiled.
    *   When evaluating a proc body, the interpreter clones the
        Rc<CommandEnum>.  The reference count will be decremented at
        the end of the call.
    *   If the proc renames itself, the lookup table is changed, but
        the definition is not, and since it's an Rc that's OK.
    *   If the proc redefines itself, the Rc in the hashmap is replaced
        by a new one; and when the proc stops executing, the old version
        is cleaned up.
    *   Woohoo!
*   Insight: Command Context
    *   A command can be passed a context struct using RefCell.  It can
        borrow it and update it as needed, so long as it itself is
        executing.
    *   Still to be answered: how to allow different commands to be
        passed different context cells?
        *   Probably need to use trait objects.
        *   A proc is already a struct.
        *   A built-in command could also be implemented as a struct.
*   Simplest approach for now:
    *   Parse and evaluate in one go.
        *   Simplifies the parser.
        *   Pretty much guarantees I get the right semantics.
    *   Write detailed tests.
    *   Then elaborate as needed.

## 2019-01-06 (Sunday)

*   Still pondering how to add context to client-defined commands.
    *   So that the command can make direct changes to the user's data.
*   At present, Interp is defined as Interp<T>, and a mutable T is passed
    to each command when it executes.  But it means that all commands
    get the same T.  This might be fine for a game, where there's a single
    context, but it's not general.
*   It might be possible to give each command its own context struct, which
    is *owned* by the Interp and can be modified by the command; and then
    queried by the owner of the Interp.

## 2019-01-05 (Saturday)

*   Created the project.
*   The first step is to parse a script into commands, and parse the commands
    into words, according to the Dodekalogue.
*   I will need an Interp to handle this.
*   Added an Interp struct with new(), define(), eval() methods, although eval()
    doesn't do anythingy yet.
*   Added molt_check_args() function.  Perhaps should be method.
    *   Not clear whether I want the Interp API to be methods or functions.
*   Realized that I needed a Context type, so that client-created extensions
    can access their context.  But I'm not at all sure how to do this
    generally; it's much trickier than in C.
