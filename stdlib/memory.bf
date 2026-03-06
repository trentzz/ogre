@fn clear {
    [-]
}

@fn clear2 {
    [-]>[-]<
}

@fn clear3 {
    [-]>[-]>[-]<<
}

@fn clear4 {
    [-]>[-]>[-]>[-]<<<
}

@fn clear5 {
    [-]>[-]>[-]>[-]>[-]<<<<
}

@fn clear_right {
    >[-]<
}

@fn swap {
    [>>+<<-]>[<+>-]>[<+>-]<<
}

@fn push_right {
    [>+<-]>
}

@fn pull_left {
    <[>+<-]
}

@fn copy_right {
    [>+>+<<-]>>[<<+>>-]<<
}

@fn copy_left {
    [<+>+<-]>>[<<+>>-]<<
}

@fn dup {
    [>+>+<<-]>>[<<+>>-]<<
}

@fn zero_range_5 {
    [-]>[-]>[-]>[-]>[-]<<<<
}

@fn rotate3 {
    [>>>+<<<-]
    >[<<+>>-]<
    >>[<<<+>>>-]<<<
    >>[<+>-]<<
}

@doc Clear 10 cells starting from the current cell. Pointer returns to cell 0.
@fn clear_range_10 {
    [-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]>[-]<<<<<<<<<
}

@doc Swap cell 0 and cell 2 (non-adjacent). Uses cell 1 as temp. Cell 1 must be 0 initially.
@fn swap_nonadj {
    [>+<-]
    >>[<<+>>-]<<
    >[>+<-]<
}

@doc Reverse cells 0, 1, 2: swap c0 and c2. Uses cell 3 as temp. Cell 1 preserved.
@fn reverse3 {
    >[>>+<<-]<
    [>+<-]>>[<<+>>-]<<>[>+<-]<
    >>>[<<+>>-]<<<
}

@doc Copy the value of cell 0 into cells 0-4 (fill 5 cells with same value).
@doc Destructive: uses cell 5 as scratch. Cell 0 value is preserved.
@fn fill_5 {
    [>+>+>+>+>+<<<<<-]
    >>>>>[<<<<<+>>>>>-]<<<<<
}

@doc Shift cells 0,1,2 right: c2=c1, c1=c0, c0=0. Old c2 discarded.
@fn shift_right_3 {
    >>[-]<<
    >[>+<-]<
    [>+<-]
}
