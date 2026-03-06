@doc Convert input to uppercase. Reads until newline or EOF.
@doc Simply subtracts 32 from each character (assumes lowercase input).
@fn do_upper {
    ,----------
    [
        ++++++++++
        --------------------------------
        .[-]
        ,----------
    ]
}

@doc Convert input to lowercase. Reads until newline or EOF.
@doc Simply adds 32 to each character (assumes uppercase input).
@fn do_lower {
    ,----------
    [
        ++++++++++
        ++++++++++++++++++++++++++++++++
        .[-]
        ,----------
    ]
}

@doc Swap case of input: uppercase becomes lowercase and vice versa.
@doc Reads until newline or EOF. Non-letter characters passed through unchanged.
@doc For each char: check if uppercase (65-90) then add 32, if lowercase (97-122) subtract 32, else print as-is.
@doc Uses cells 0-5 as workspace.
@fn do_swap_case {
    ,----------
    [
        ++++++++++

        is uppercase check: copy c0 to c3 via c4
        [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
        c3 subtract 65
        >>>-----------------------------------------------------------------<
        c2 = 26
        ++++++++++++++++++++++++++
        bounded loop on c2
        [
            >
            [>+>+<<-]>>[<<+>>-]<<
            >
            >+<
            [>-<[-]]
            >
            [<<<<+>>>>-]
            <<-
            <-
        ]
        >[-]<
        <<

        c1 = 1 if uppercase else 0 and c0 still original
        >[
            -
            <
            ++++++++++++++++++++++++++++++++
            .[-]
            >
        ]<

        if c0 still nonzero not uppercase check lowercase
        [
            is lowercase: copy c0 to c3 via c4
            [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
            c3 subtract 97
            >>>-------------------------------------------------------------------------------------------------<
            c2 = 26
            ++++++++++++++++++++++++++
            bounded loop on c2
            [
                >
                [>+>+<<-]>>[<<+>>-]<<
                >
                >+<
                [>-<[-]]
                >
                [<<<<+>>>>-]
                <<-
                <-
            ]
            >[-]<
            <<

            c1 = 1 if lowercase
            >[
                -
                <
                --------------------------------
                .[-]
                >
            ]<

            not a letter print as is
            [.[-]]
        ]

        clean scratch cells
        >[-]>[-]>[-]>[-]>[-]<<<<<

        ,----------
    ]
}

@fn do_ascii_to_decimal {
    ,----------
    [
        ++++++++++

        divmod c0 by 10
        >++++++++++<
        [->-[>+>>]>[+[-<+>]>+>>]<<<<<]

        move quotient c3 to c6
        >>>[->>>+<<<]<<<

        set c7 to 10
        >>>>>>>++++++++++<<<<<<<

        divmod c6 by 10
        >>>>>>[->-[>+>>]>[+[-<+>]>+>>]<<<<<]<<<<<<

        at c0 now c2=ones c8=tens c9=hundreds

        print hundreds and set flag at c10
        >>>>>>>>>
        [
            ++++++++++++++++++++++++++++++++++++++++++++++++
            .[-]
            >+<
        ]

        check flag at c10 for tens
        <>>[
            -<<
            ++++++++++++++++++++++++++++++++++++++++++++++++
            .[-]
            >>
        ]<<

        print tens if nonzero and flag was not set
        [
            ++++++++++++++++++++++++++++++++++++++++++++++++
            .[-]
        ]

        print ones at c2
        <<<<<<
        ++++++++++++++++++++++++++++++++++++++++++++++++
        .[-]

        print space
        ++++++++++++++++++++++++++++++++.[-]

        clean c1 c7 c10
        <[-]>>>>>>[-]>>>[-]<<<<<<<<<<

        ,----------
    ]
}

@fn do_decimal_to_ascii {
    >
    ,----------
    [
        ++++++++++
        --------------------------------

        >+<

        [
            >-<
            ----------------

            move digit to c3
            [->>+<<]

            multiply c0 by 10
            <[->++++++++++<]
            >[-<+>]

            add digit from c3 to c0
            >>[-<<<+>>>]<<
        ]

        >[
            -
            <<.[-]
            >>
        ]

        <

        ,----------
    ]

    <.[-]
}
