@fn print_A {
    +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]
}

@fn print_B {
    ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]
}

@fn print_exclaim {
    +++++++++++++++++++++++++++++++++.[-]
}

@fn print_dash {
    +++++++++++++++++++++++++++++++++++++++++++++.[-]
}

@fn print_colon {
    ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.[-]
}

@doc Subtract 32 from cell 0 (lowercase to uppercase)
@fn to_upper {
    --------------------------------
}

@doc Add 32 to cell 0 (uppercase to lowercase)
@fn to_lower {
    ++++++++++++++++++++++++++++++++
}

@doc Set cell 1 to 1 if cell 0 is a digit (ASCII 48-57). Non-destructive on c0.
@doc Uses cells 1-5 as scratch. Cells 1-5 cleaned after.
@fn is_digit {
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    ------------------------------------------------
    <
    ++++++++++
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
}

@doc Set cell 1 to 1 if cell 0 is space (32). Nondestructive.
@fn is_space {
    [>+>+<<-]>>[<<+>>-]<
    --------------------------------
    >+<
    [>-<[-]]>
    [<+>-]<
    <
}

@doc Add 48 to cell 0 (numeric value to ASCII char)
@fn digit_to_char {
    ++++++++++++++++++++++++++++++++++++++++++++++++
}

@doc Subtract 48 from cell 0 (ASCII char to numeric value)
@fn char_to_digit {
    ------------------------------------------------
}

@doc Set cell 1 to 1 if cell 0 is uppercase (A-Z, ASCII 65-90). Non-destructive on c0.
@doc Uses cells 1-5 as scratch. Cells 1-5 cleaned after.
@fn is_upper {
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    -----------------------------------------------------------------
    <
    ++++++++++++++++++++++++++
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
}

@doc Set cell 1 to 1 if cell 0 is lowercase (a-z, ASCII 97-122). Non-destructive on c0.
@doc Uses cells 1-5 as scratch. Cells 1-5 cleaned after.
@fn is_lower {
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    -------------------------------------------------------------------------------------------------
    <
    ++++++++++++++++++++++++++
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
}

@doc Set cell 1 to 1 if cell 0 is a letter (A-Z or a-z). Non-destructive on c0.
@doc Uses cells 1-6 as scratch. Checks uppercase then lowercase, ORs results.
@fn is_alpha {
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    -----------------------------------------------------------------
    <
    ++++++++++++++++++++++++++
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
    >[>>>>>+<<<<<-]<
    >>[-]>[-]>[-]>[-]<<<<<
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    -------------------------------------------------------------------------------------------------
    <
    ++++++++++++++++++++++++++
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
    >>>>>>[-<<<<<+>>>>>]<<<<<<
    >[[-]>+<]>[<+>-]<<
}

@doc Set cell 1 to 1 if cell 0 is newline (ASCII 10). Non-destructive on c0.
@doc Uses cells 1-2 as scratch.
@fn is_newline {
    [>+>+<<-]>>[<<+>>-]<
    ----------
    >+<
    [>-<[-]]>
    [<+>-]<
    <
}

@doc Set cell 1 to 1 if cell 0 is printable ASCII (32-126). Non-destructive on c0.
@doc Uses cells 1-5 as scratch. Cells 1-5 cleaned after.
@fn is_printable {
    [>>>+>+<<<<-]>>>>[<<<<+>>>>-]<<<<
    >>>
    --------------------------------
    <
    +++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
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
}

@doc Print a digit 0-9 from cell 0 (adds 48 to get ASCII, prints, clears cell)
@fn print_digit {
    ++++++++++++++++++++++++++++++++++++++++++++++++.[-]
}
