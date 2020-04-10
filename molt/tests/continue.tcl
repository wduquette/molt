# Test Script: continue
#
# NOTE: continue is primarily tested in the context of the various loop commands.

test continue-1.1 {continue command} {
    set code [catch continue result opts]
    list $code $result $opts
} -ok {4 {} {-code 4 -level 0}}
