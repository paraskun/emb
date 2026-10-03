set ::gw_fifo "out/gtkwave.fifo"

proc gw_recv {} {
    while {[gets $::gw_fd line] >= 0} {
        if {$line ne ""} { catch {uplevel #0 $line} }
    }
}

set ::gw_fd [open $::gw_fifo {RDWR}]
fconfigure $::gw_fd -blocking 0
fileevent $::gw_fd readable gw_recv
