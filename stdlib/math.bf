@fn zero {
    [-]
}

@fn inc {
    +
}

@fn dec {
    -
}

@fn inc5 {
    +++++
}

@fn dec5 {
    -----
}

@fn inc10 {
    ++++++++++
}

@fn dec10 {
    ----------
}

@fn double {
    [>++<-]>[<+>-]<
}

@fn triple {
    [>+++<-]>[<+>-]<
}

@fn multiply_by_10 {
    [>++++++++++<-]>[<+>-]<
}

@fn divmod_10 {
    >++++++++++<
    [->-[>+>>]>[+[-<+>]>+>>]<<<<<]
    >[<+>-]<
}

@fn add_to_next {
    [>+<-]
}

@fn move_right {
    [>+<-]>
}

@fn move_left {
    [<+>-]<
}

@fn copy_right {
    [>+>+<<-]>>[<<+>>-]<<
}

@fn is_zero {
    [>+>+<<-]>>[<<+>>-]<
    >+<
    [>-<[-]]>
    [<+>-]<
    <
}

@fn is_nonzero {
    [[-]>+<]
}

@fn negate {
    >++++++++[<++++++++++++++++++++++++++++++++>-]<
    [>+<-]
    >[-<->]<
    [-]
    >>[<<+>>-]<<
}

@fn abs_diff {
    [->-<]>
    [<+>-]<
}

@fn min {
    >>[<<+>>-]<<
    [>+>+<<-]
    >>[<<+>>-]<<
    >[<->-]<
    [>>+<<[-]]
    >[-]>
    [<<<+>>>-]<<<
}

@fn max {
    >>[<<+>>-]<<
    [>+>+<<-]
    >>[<<+>>-]<<
    >[<->-]<
    [>>+<<[-]]
    >>
    [<<<+>>>-]<<<
    >[<+>-]<
}

@doc Square cell 0 (c0 = c0 * c0). Uses cells 1-4 as scratch.
@fn square {
    [>+>+<<-]>>[<<+>>-]<<
    >[<[>>+>+<<<-]>>>[<<<+>>>-]<<-]
    >[<+>-]<
    >[-]>[-]<<
}

@doc Compute c0 mod c1, result in c0. Cell 1 zeroed. Uses cells 2-4 as scratch.
@fn modulo {
    [->-[>+>>]>[+[-<+>]>+>>]<<<<<]
    >[-]>[<<+>>-]<<
    >>[-]>[-]<<<
}

@doc Multiply c0 by c1, result in c0. Cell 1 zeroed. Uses cell 2 as scratch.
@fn multiply {
    >[<[>>+>+<<<-]>>>[<<<+>>>-]<<-]
    >[<+>-]<
    >[-]>[-]<<
}

@doc Set cell 0 to 1 if nonzero, 0 if zero (boolean cast / is_positive)
@fn is_positive {
    [[-]>+<]>[<+>-]<
}

@doc Clamp c0 between c1 (min) and c2 (max). Result in c0.
@doc Uses cells 3-6 as scratch. c1 and c2 zeroed after.
@fn clamp {
    [>>>+<<<-]
    >[>>>>+<<<<-]>
    [>>>>>+<<<<<-]
    <<<
    >>>
    [>+>+<<-]>>[<<+>>-]<
    [<->-]<
    [[-]
        >>[-]<<
        <<<[>>>+<<<-]>>>
    ]
    >
    [<<<+>>>-]
    <<<
    >>>[-]<<<
    >[>>>>+<<<<-]>>>>
    [<+<+>>-]<<[>>+<<-]>
    [<->-]<
    [[-]
        >>>[-]<<<
        >>[<<<+>>>-]<<<
    ]
    >>
    [<<<+>>>-]
    <<<
    >[-]>[-]>[-]>[-]>[-]>[-]<<<<<<<
    >[<+>-]<
}
