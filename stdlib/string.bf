@doc Read and discard one character from stdin. Cell 0 cleared.
@fn skip_char {
    ,[-]
}

@doc Read and discard whitespace from stdin until a non-space char is found.
@doc The non-space char remains in cell 0. Uses cells 1-2 as scratch.
@doc If input is all spaces, cell 0 will be 0 (EOF).
@fn skip_spaces {
    ,
    [>+>+<<-]>>[<<+>>-]<
    --------------------------------
    >+<[>-<[-]]>[<+>-]<
    [[-]<[-],
        [>+>+<<-]>>[<<+>>-]<
        --------------------------------
        >+<[>-<[-]]>[<+>-]<
    ]
    [-]<
}

@doc Read and discard characters until newline (10) or EOF (0). Newline consumed.
@doc Cell 0 is 0 after. Uses cell 1 as scratch.
@fn skip_line {
    ,
    >++++++++++<
    [>-<-]>
    [<+>-]<
    [
        [-],
        >++++++++++<
        [>-<-]>
        [<+>-]<
    ]
    [-]
}

@doc Read characters from input until newline (10) or EOF (0), storing in successive cells.
@doc Newline is not stored. Pointer ends at the zero terminator (one past last char).
@fn read_line {
    ,----------[++++++++++>,----------][-]
}

@doc Read characters from input until space (32) or EOF (0), storing at c0 onward.
@doc Each char is stored in successive cells. Pointer ends at the zero terminator.
@doc Note: only space is treated as word delimiter. Use read_line for newline-terminated input.
@fn read_word {
    ,
    [
        --------------------------------
        [
            ++++++++++++++++++++++++++++++++
            >,
            --------------------------------
        ]
    ]
    [-]
}

@doc Print cells starting from c0 until a zero cell is hit. Advances right.
@doc Pointer ends at the zero-terminator cell.
@fn print_string {
    [.>]
}

@doc Read one char from input. Set cell 1 to 1 if it matches cell 0, else 0.
@doc Cell 0 preserved. Uses cells 2-3 as scratch.
@fn compare_char {
    [>>+>+<<<-]>>>[<<<+>>>-]<<<
    >,
    [>-<-]>
    >+<[>-<[-]]>
    [<<+>>-]<[-]<<
}

@doc Read decimal digits from input into cell 0. Stops at non-digit.
@doc Accumulates digits: for "123", cell 0 = 123. Uses cells 1-3 as scratch.
@doc The non-digit terminator is consumed and discarded.
@fn read_decimal {
    [-]
    ,
    ------------------------------------------------
    [>+>+<<-]>>[<<+>>-]<
    ++++++++++
    [<->-]<
    [[-]
        ++++++++++
        ------------------------------------------------
        >++++++++++<
        [->+<]
        >[->++++++++++<]>[-<<+>>]<<<
        ,
        ------------------------------------------------
        [>+>+<<-]>>[<<+>>-]<
        ++++++++++
        [<->-]<
    ]
    >[-]>[-]<<
}
