# GCL Development Journal

### 2018-01-13 (Sunday)
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
                gcl::eval, as I can't decrement the stack depth.
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

## 2018-01-09 (Wednesday)
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

## 2018-01-08 (Tuesday)
*   Revised to use command storage and traits as described in yesterday's
    journal entry.
*   Added a rustyline-based shell, but it doesn't actually call the interpreter
    yet.

## 2018-01-07 (Monday)

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

## 2018-01-06 (Sunday)

*   Still pondering how to add context to client-defined commands.
    *   So that the command can make direct changes to the user's data.
*   At present, Interp is defined as Interp<T>, and a mutable T is passed
    to each command when it executes.  But it means that all commands
    get the same T.  This might be fine for a game, where there's a single
    context, but it's not general.
*   It might be possible to give each command its own context struct, which
    is *owned* by the Interp and can be modified by the command; and then
    queried by the owner of the Interp.

## 2018-01-05 (Saturday)

*   Created the project.
*   The first step is to parse a script into commands, and parse the commands
    into words, according to the Dodekalogue.
*   I will need an Interp to handle this.
*   Added an Interp struct with new(), define(), eval() methods, although eval()
    doesn't do anythingy yet.
*   Added gcl_check_args() function.  Perhaps should be method.
    *   Not clear whether I want the Interp API to be methods or functions.
*   Realized that I needed a Context type, so that client-created extensions
    can access their context.  But I'm not at all sure how to do this
    generally; it's much trickier than in C.
