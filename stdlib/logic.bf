@fn not {
    >+<[>-<[-]]>[<+>-]<
}

@fn bool {
    [[-]>+<]>[<+>-]<
}

@fn and {
    >[-]<
    [>+<[-]]
    >[<+>-]<
}

@fn or {
    >[<+>-]<
    [[-]>+<]>[<+>-]<
}

@fn equal {
    [>>+<<-]
    >[>>+<<-]<
    >>>[<->-]<<<
    >>
    >+<[>-<[-]]>[<+>-]<
    [-<<+>>]<<
    >[-]>[-]<<
}

@doc Exclusive OR of cell 0 and cell 1. Result in cell 0. Cell 1 zeroed.
@doc Assumes boolean inputs (0 or 1). Result: 1 if exactly one input is nonzero.
@fn xor {
    >[<+>-]<
    -
    >+<[>-<[-]]>[<+>-]<
    >[-]<
}

@doc NAND of cell 0 and cell 1. Result in cell 0. Cell 1 zeroed.
@doc Assumes boolean inputs (0 or 1). Uses cell 2 as scratch.
@fn nand {
    [>[>>+<<-][-]]>[-]>[<<+>>-]<<
    >+<[>-<[-]]>[<+>-]<
    >[-]>[-]<<
}

@doc If cell 0 is nonzero, result is cell 1; else result is 0. Result in cell 0.
@doc Cell 0 used as condition (zeroed). Cell 1 zeroed.
@fn if_nonzero {
    [>>+<<-]
    >>[[-]<[<+>-]>]<<
    >[-]>[-]<<
}

@doc Set cell 0 to 1 if c0 > c1, else 0. Both cells consumed.
@doc Uses cells 2-5 as scratch. Correct for all unsigned 8-bit values.
@fn greater_than {
    [>>+<<-]
    >[>>+<<-]<
    >>[
        >
        [>+>+<<-]>>[<<+>>-]<<
        >
        >+<
        [>-<[-]]
        >
        [<<<<<+>>>>>-]
        <<-
        <-
    ]
    >[-]>[-]>[-]<<<
    <<
}

@doc Set cell 0 to 1 if c0 < c1, else 0. Both cells consumed.
@doc Uses cells 2-5 as scratch. Correct for all unsigned 8-bit values.
@fn less_than {
    >[>+<-]<
    [>>>+<<<-]
    >>[
        >
        [>+>+<<-]>>[<<+>>-]<<
        >
        >+<
        [>-<[-]]
        >
        [<<<<<+>>>>>-]
        <<-
        <-
    ]
    >[-]>[-]>[-]<<<
    <<
}
