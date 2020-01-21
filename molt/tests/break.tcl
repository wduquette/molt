# Test Script: break command
#
# NOTE: break is primarily tested in the context of the various loop commands.

test break-1.1 {break command} {
    set code [catch break result opts]
    list $code $result $opts
} -ok {3 {} {-code 3 -level 0}}
