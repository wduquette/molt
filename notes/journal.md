# Molt Development Journal

Things to remember to do soon:

*   Implement Debug for Value.  Should output a pair, `Value[string_rep,data_rep]`, or
    something like that.
*   There are a bunch of internal "parse_*" routines in interp.rs that
    return `MoltResult` but should possibly return `Result<String,ResultCode>`.
*   MoltList should maybe be a newtype with helper methods.

### 2019-06-22 (Saturday)
*   Began implementing From<T> for the Value input types.
    *   String, &str, bool, int, float.
    *   Works a treat; both `Value::from(x)` and `let val: Value = x.into()` work as expected.

### 2019-06-19 (Wednesday)
*   Finished updating the molt:: code so it compiles.
    *   Bodies of many commands are compiled out at present.
*   One test failure, interp::tests::test_recursion_limit, because the "proc" command is
    currently FUBAR.

### 2019-06-18 (Tuesday)
*   Converted scope.rs to use MoltValue.
*   Renamed expr::Value (an internal type) to expr::Datum.
*   Renamed value::MoltValue to value::Value.
*   Tried to make ResultCode::Error(String) be ResultCode::Error(Value).
    *   Problem: ResultCode current implements Eq and PartialEq.  Value doesn't.
*   Implemented Eq and PartialEq (comparing the string_reps).
*   Updated all types in types.rs to use Value where appropriate.
*   Updated list.rs accordingly.
*   Updated expr.rs accordingly.
*   Still lots of work in interp.rs and commands.rs to go.

### 2019-06-17 (Monday)
*   Fleshed out the MoltValue::from_list and MoltValue::as_list methods, and added list formatting
    to Datum's Display implementation.
*   Added examples to the doc comments for the MoltValue from_str, from_string, as_string, from_bool,
    as_bool, from_int, as_int, from_float, and as_float methods.

### 2019-06-16 (Sunday)
*   Fixed up the MoltValue::as_int and MoltValue::as_float methods
    *   as_int parses using the Interp::parse_int algorithm.
    *   Both methods use ResultCode.
*   Added MoltValue::from_bool and MoltValue::as_bool, using ResultCode and the
    Interp::get_int algorithm for parsing.

### 2019-06-15 (Saturday)
*   Copied the new MoltValue type from the wduquette/value repo to this code base.
    *   It isn't yet used for anything, but it's now available for integration.

### 2019-06-01 (Saturday)
*   Handling floating point errors:
    *   Per KBK, the floating point subsystem just handles floating point overflow and
        underflow.
        *   On underflow, values are "denormalized" and then go to 0.
            *   "denormalized": I presume this means that you trade mantissa bits for exponent
                bits.
        *   On overflow, you get Infinity, -Infinity.
        *   On other errors you get NaN.
        *   Rust has methods to check for all of these.
    *   In the current expr, I could:
        *   Extend my parsing/formatting to allow Inf, -Inf, and throw errors on NaN.
        *   -OR- throw errors on all three.
        *   The latter makes the most sense.
    *   I can do better after I've got some kind of MoltValue in place so that floats remain
        floats.
*   Working on a MoltValue prototype in the "value" repo.

### 2019-05-27 (Monday)
*   Integer Division results:
    *   I tried Rust's euclidean division gives the same result as the
        normal division for this problem; it isn't Rust's version of the
        TCL way.
    *   Here's what I get for a series of problems.

| Operation | TCL 7.6 | TCL 8.6 | Molt | Rust |
| --------: | ------: | ------: | ---: | ---: |
| 12 / 10   |  1      |  1      |  1   |  1   |
| 12 % 10   |  2      |  2      |  2   |  2   |
| -12 / 10  | -2      | -2      | -1   | -1   |
| -12 % 10  |  8      |  8      | -2   | -2   |
| 12 / -10  | -2      | -2      | -2   | -1   |
| 12 % -10  | -8      | -8      | -8   |  2   |
| -12 / -10 |  1      |  1      |  2   |  1   |
| -12 % -10 | -2      | -2      |  8   | -2   |

    *   Apparently C reliably rounds down if the numerator is negative and the denominator is
        positive, whereas Rust always rounds to zero.
        *   Since the Tcl 7.6 code does nothing special in that case, we get the Rust behavior,
            not the C behavior.
    *   Sent a note to Kevin Kenny about this: by principle of least surprise, should Molt
        support Rust behavior (so that the application and its script get the same answer)
        or TCL behavior?
        *   Rust behavior is dead easy; and the results show that Rust isn't simply doing what
            the platform does, but is doing what Rust does.
        *   I'd have to add some logic to make the Molt version look like TCL.
*   Per Kevin Kenny, Fortran, C, and many other languages have now standardized on rounding to
    zero; and the floating point standard says that floats are to be rounded toward zero in
    similar circumstances.  So it's reasonable to do that in Molt.
*   Stack Traces
    *   Where is there stack-trace code in TCL 7.6?
        *   See the end of Tcl_Eval()
            *   On error, it calls Tcl_AddErrorInfo with a line of data, one of the following,
                where "{command}" is the command being executed, possibly elided.
                *   `\n    while executing\n"{command}"` (if this function detects an error)
                *   `\n    invoked from within\n"{command}"` (if a called function returned an error)
        *   The error info is saved in the interp; it's really an extension of the result.
        *   It's only saved while processing genuine error returns.  The logic is
            straightforward:
            *   If you detect an error, AddErrorInfo, then return the error
            *   If you receive an error, add your own AddErrorInfo and return the error.
        *   Tcl_AddErrorInfo is defined in tclBasic.c, and used in many different places.
    *   For Rust, it would make more sense to replace `ResultCode::Error(String)` with   
        `ResultCode::Error(ErrorInfo), where ErrorInfo is a struct that can accumulate the error
        trace information.
        *   Or, perhaps, `ResultCode::Error(String,ErrorInfo)`
        *   We really need to do that anyway if we want to support the full TCL 8 return syntax.
            *   But need dicts first for all that?
    *   Found all uses of AddErrorInfo in TCL 7.6; see stack_trace.md.

### 2019-05-25 (Saturday)
*   Updated to latest Rust.
*   Fix issue #14: Evaluation stack depth checking
    *   Added nesting level tracking to Interp::eval().
    *   All existing tests pass.
    *   Added test to interp.rs to show that we catch an infinite loop.
    *   Added `molt test`, interp.tcl.
    *   Updated default max nesting limit to 1000 from 255, which is the
        Tcl 7.6/8.6 default.
    *   To my surprise, there's no Tcl-level interface to set the max
        nesting level in either Tcl 7.6 or 8.6.  I thought there was.
*   Added Interp::recursion_limit, set_recursion_limit.
*   Fixed new rustc warnings in scope.rs.

### 2019-04-27 (Saturday)
*   Updated to latest Rust.
*   Merged last month's API work.  Still a lot left to do.
*   Defined Interp::subst_backslashes() method
    *   Retained the existing subst_backslashes() function as a `pub(crate)`, as it's
        convenient for list parser not to require the interp handle.
    *   The method delegates to the function.
    *   Added doc comment and test.

### 2019-03-21 (Thursday)
*   Added some more Rustdoc to interp.rs.
*   Added lots of Rustdoc to types.rs.
*   MoltValue Thoughts
    *   A MoltValue should have at least one of:
        *   An optional string rep
        *   An optional internal rep.
        *   It must be possible to register new internal representation types.
            (MoltTypes)
    *   A MoltType:
        *   Should be an efficient internal representation for some kind of
            data (list, dict)
        *   Should be able to retrieve itself from a MoltValue that contains
            an internal rep of its type and provide it to Rust code.
            *   E.g.,
        *   Should be able to produce its own string representation.
        *   Should be able to parse its own string representation
```rust
    impl MoltType for MoltList {
        pub fn from(val: MoltValue) -> Result<MoltList,ResultCode> {...}
        pub fn parse(string: &str) -> Result<MoltList,ResultCode> {...}
        pub fn to_string(&self) -> String {...}
    }
```
    *   Questions:
        *   How does the `MoltValue` contain the instance of the MoltType?
            *   As `Option<dyn MoltType>`?  I.e., MoltList implements MoltType.
        *   How does the `from` method safely determine whether the `MoltValue`
            contains a MoltList?  
            *   Is there something like instanceof?
            *   Looks like the "Any" trait is what I'm looking for.  See
                module std::any.
    *   Using std::any:
        *   Define trait MoltType, which has the methods needed to convert
            a value to and from a string.
        *   MoltValue has a String and a dyn Any + MoltType + Clone.
        *   As an Any, it can be converted back to the type I want, getting
            Some or None.

### 2019-03-20 (Wednesday)
*   Deleted `vec_string_to_str` function, as it's not used.
*   Consider whether to define the Command trait using AsRef so that commands
    can take &[String] or &[&str].  
    *   Makes them more flexible.
    *   We DO need to convert the commands we parse (which are currently
        Vec<String>) into Vec<&str> before passing them to the Command handler.
    *   Command handlers are called only by eval_context.
    *   Maybe define them as taking &[String], since that's what they really
        are?  Then no conversion is given.
    *   What I really need to think about here is what MoltValue should look
        like, since that's where I'm going.


### 2019-03-19 (Tuesday)
*   Began work on revising the public API.

### 2019-03-16 (Saturday)
*   VarStack and StackFrames
    *   It occurs to me that the VarStack is exactly the set of
        stack frames, as regards `upvar`, `uplevel`, and `info level`,
        but not as regards the stack trace.
    *   The Standard TCL stack trace is far more detailed.

### 2019-03-03 (Sunday)
*   Added a bunch of issues to GitHub.
*   Added `rename` command.
    *   Added tests.
    *   Revised existing tests to cleanup temporary procs.
    *   Added man page.
*   Added minimal `error` command.
    *   And test and man page.
*   Added `while` command.
    *   And test and man page.

### 2019-03-02 (Saturday)
*   Workspace Architecture
    *   Trying out a multi-crate workspace in workspace-arch branch.
        *   `molt`: Library crate, the core language.
        *   `molt-shell`: Library crate, the REPL and test harness.
        *   `molt-app`: The command-line application, giving access to the REPL and test
            harness.
    *   Done, and it works.
        *   The main.rs in `molt-app` is dirt simple, which is appropriate as it's intended
            to serve as an example.
        *   `molt-shell` provides the features needed by `molt-app` that shouldn't be
            in `molt` itself.
            *   In particular, `molt` itself no longer has any external dependencies.

### 2019-03-01 (Friday)
*   Wrote test suite for `for` command.
    *   Fixed bugs:
        *   `expr` should not propagate "continue" and "break".
        *   `break` is allowed in the "next" script, but "continue" isn't.

### 2019-02-28 (Thursday)
*   Fixed errors in `for`.
*   Fixed output errors in `test`
*   `test` now pushes a var scope before executing the test body, and pops after, so that
    each test body always gets a clean scope.
*   Tried duplicating the Tcl 7.6 lappend tests.
    *   lappend works differently in the corner cases than it used it.
    *   I'm going to need to work with the Tcl 8 test suite instead.
*   Added the Tcl 8.6 lappend tests one at a time.
    *   Found three errors in list_to_string()
        *   unmatched left-braces trigger the escape code, but it wasn't escaping them.
        *   The error messages for unmatched braces and quotes were wrong.
        *   All fixed.
    *   There are many of these tests (append-7.* and following) that involve traces
        and the `{*}` operator that I didn't bother copying.
*   The existing `expr` and `global` tests had some failures because test bodies are no longer
    in the global scope.
    *   Fixed.

### 2019-02-27 (Wednesday)
*   Implemented `for`

### 2019-02-24 (Sunday)
*   The `source` command.
    *   Checked the TCL docs and wrote a couple of test scripts.
    *   `source` does not change the current working directory in any way.
    *   `source` reads up to the first ^Z, allowing for scripted documents.
*   As a result:
    *   "molt test" needs to document that it changes the CWD to the
        folder containing the test script being executed.
    *   The Molt Book needs to document these differences.
    *   DONE.
*   Wrote the first chapter of the Molt Book, and updated the command line
    section.
*   Simplified the molt/README.md, and references the Molt Book.

### 2019-02-23 (Saturday)
*   Current `expr` status:
    *   "eq", "ne", variables, commands, quoted and braced strings are not
        tested.
    *   Parentheses are not tested.
    *   Precedence is not tested.
    *   Not all of the math functions are handled.
    *   floating point/integer error handling isn't yet handled.  There must be a way to
        trap overflow/underflow, etc. without panicking, but I haven't looked into that
        in Rust yet.
        *   For integers: http://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/
        *   See std:f64.  Provides NAN, INFINITY, etc.
*   Realized that I'm not handling "no_eval" correctly in the code I added
    on Thursday.  If the parse method uses no_eval, I need to set the context
    to the expression parser's no_eval.
    *   Done, and it works.
*   Integer/floating point error handling:
    *   What I'd like to do, I think, is get a hold of the TclTest scripts
        for `expr` before I spend too much time on the numerics.
    *   But this means having a reasonably TclTest compatible test tool.
    *   This applies to a lot of the expr testing, actually.
    *   Does Tcl 7.6 have TclTest?
        *   Checked; it has a precursor, and a bunch of expr tests.  I'll
            want to make use of those.
*   "in" and "ni" are now evaluated.
*   Added math funcs abs(), int(), round(), double().
*   Merged the expr-parser branch.
*   Added `expr` to the Molt Book.
*   Added "molt shell", "molt test".
    *   "molt test" is the test harness, and accumulates test results.
*   Added a "description" field to the `test` command.
*   Add "source" command.
*   Added test/all.tcl, to run all of the test scripts.
    *   Apparently have some cross-talk between test/commands.tcl and test/expr.tcl
        *   We get some test failures if we run the expr.tcl tests after the commands.tcl
            tests.
    *   Updated test_harness.rs to set the current working directory to the file's directory
        when sourcing the test script, so that source commands will work right.

### 2019-02-21 (Thursday)
*   lappend command
    *   I wanted this for expr testing.
*   Added "lexpr" to test_expr.tcl, and simplified all of the tests accordingly.
    *   Also, added some more floating point and mixed integer/floating point tests.
    *   No new errors found.
*   Added the "?:" operator, with test.
*   Was looking at the function lexing, expecting to find support for boolean constants,
    and didn't.  Apparently, the Tcl 7.6 expression parser doesn't support them.
    Seems like it would be easy enough to add, though.
    *   In ExprMathFunc, where it looks to see if the next token is a "(", first look to see
        if the string is a boolean constant (or, in fact, one of "eq", "ne", "in", "ni").
        If it is, return the appropriate value.
*   Added eq, ne, in, and ni to the list of token types, the precedence table, etc.
    *   All four get lexed.
    *   Evaluation of "in" and "ni" returns a "not yet implemented" error.
    *   "eq" and "ne" appear to work with numeric arguments.
        *   I can't yet enter non-numeric arguments.
*   Added handling for:
    *   interpolated variables
    *   interpolated commands
    *   quoted strings
    *   braced strings
*   At some point I should see about making interp.rs use CharPtr instead of Context, leaving
    the parsing context an internal struct in interp.rs the way it is in expr.rs.
*   Current status:
    *   "in" and "ni" are parsed but not evaluated.
        *   There's no point until I can handle variables or strings.
    *   expr_lex() also doesn't handle math functions.  That will be a lot of work, but
        it should be straightforward at this point.
    *   floating point/integer error handling isn't yet handled.  There must be a way to
        trap overflow/underflow, etc. without panicking, but I haven't looked into that
        in Rust yet.
        *   For integers: http://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/
        *   See std:f64.  Provides NAN, INFINITY, etc.

### 2019-02-20
*   Expression Parsing.
    *   Added tests for all of the existing operators.
        *   ?: isn't yet implemented.
        *   && had a bug, and || wasn't implemented.  Fixed both problems.
        *   true, false, etc., are not yet accepted as valid literals.
            *   I suspect this is in the part of the lexer that handles functions.

### 2019-02-18
*   Expression Parsing.
    *   At some point I'm going to need to figure out how you get info about floating-point
        errors in Rust, e.g., overflow, underflow.
    *   I have expr_lex working (apparently) for numbers and operators.
        *   Still need to handle interpolated variables, commands, etc.
    *   I have expr_get_value working partially; I've got the skeleton complete, but all
        of the operators return a "not yet implemented" error.
        *   Using this, it appears that the lexer is working right.
    *   Note: I'm not sure my ValueType enum is the right way to do things; it seems to
        be leading to more complex code.
        *   Yup.  Switched to a simple enum plus a struct with several value fields.  Code is
            now shorter.
*   Current status:
    *   Basic math appears to be working, though much more testing is required.
    *   expr_lex() doesn't yet handle the following constructs.  In order to do so, I need to
        unify how expr.rs and interp.rs do parsing (i.e., make interp use CharPtr).
        *   interpolated variables
        *   interpolated commands
        *   quoted strings
        *   braced strings
    *   expr_lex() also doesn't handle math functions.  That will be a lot of work, but
        it should be straightforward at this point.
    *   expr_get_value() does everything but "?:".
    *   floating point/integer error handling isn't yet handled.  There must be a way to
        trap overflow/underflow, etc. without panicking, but I haven't looked into that
        in Rust yet.
        *   For integers: http://huonw.github.io/blog/2016/04/myths-and-legends-about-integer-overflow-in-rust/
        *   See std:f64.  Provides NAN, INFINITY, etc.


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
*   Tcl 7.6's Value and ParseValue
    *   Just realized: I will eventually need to preserve the string value of any parsed integers
        and doubles, because I might need it for "eq" and "ne".
*   Added lib::get_float(), and added tests for lib::get_float() and lib::get_int().

### 2019-02-16
*   Expression Parsing.
*   Starting with Tcl 7.6 parser, per Don Porter.
*   The expression parser often needs to look ahead, i.e., try to parse an integer.
    If it succeeds, the ExprContext needs to get updated to point to the next thing.
    We can do this by cloning the iterator.
    *   info.chars points at the next character in the input.
    *   let mut p = info.chars.clone() gives us the ability to work farther along.
    *   info.chars = p updates the pointer to the next thing.
*   Wrote CharPtr, a simple struct that wraps the peekable iterator and makes it work more
    like a `char* p`;
*   Added and tested `expr_looks_like_int()`.
*   Next Steps:
    *   expr_lex() needs routines that parse an unsigned long or double out of a string,
        leaving whatever's left in place.  Not clear how to do that.  The `&str.parse()`
        method doesn't do that.
*   Note: you can't easily compare two iterators for equality the way you can
    compare two `char*`'s.  Better approach: ask parsing routine to return
    `Result<Option<_>,ResultCode>`.  Keep the cloned iterator if Some, and
    not if None.
    *   Could define CharPtr to keep the input `&str` and use `enumerate()` to
        get the index of each character.  Then I could compare these for
        equality.
    *   Tried this; it appears to work.  I'd rather avoid it, though.
        Took the changes out.

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
