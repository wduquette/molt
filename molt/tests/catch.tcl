# Test Script: catch command

test catch-1.1 {catch no args} {
    catch
} -error {wrong # args: should be "catch script ?resultVarName? ?optionsVarName?"}

test catch-2.1 {catch ok} {
    catch {set a 1}
} -ok {0}

test catch-2.2 {catch error} {
    catch {error "abc"}
} -ok {1}

test catch-2.3 {catch return} {
    catch {return "abc"}
} -ok {2}

test catch-2.4 {catch break} {
    catch {break}
} -ok {3}

test catch-2.5 {catch continue} {
    catch {continue}
} -ok {4}

test catch-3.1 {catch ok value} {
    catch {set a "abc"} myvar
    set myvar
} -ok {abc}

test catch-3.2 {catch error value} {
    catch {error "def"} myvar
    set myvar
} -ok {def}

test catch-3.3 {catch return} {
    catch {return "ghi"} myvar
    set myvar
} -ok {ghi}

test catch-4.4 {catch break} {
    catch {break} myvar
    set myvar
} -ok {}

test catch-4.5 {catch continue} {
    catch {continue} myvar
    set myvar
} -ok {}

test catch-5.1 {catch options} {
    set a [catch { throw CODE "Message" } result opts]
    list $a $result [dict get $opts -code] [dict get $opts -level] [dict get $opts -errorcode]
} -ok {1 Message 1 0 CODE}
