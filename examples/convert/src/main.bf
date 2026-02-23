@import "../lib/parse.bf"
@import "../lib/convert.bf"

@call skip_dashes

read mode char into c1 to preserve it
>,<

skip remaining flag chars
@call skip_to_space

move mode from c1 to c0
>[-<+>]<

c0 has mode char: a=97 d=100 l=108 u=117

subtract 97 to check for a
-------------------------------------------------------------------------------------------------

copy c0 to c1 via c2 nondestructively
[>+>+<<-]>>[<<+>>-]<<

check if c1 is zero at c1
>>+<[>-<[-]]

dispatch ascii to decimal if matched
>[
    -
    <<[-]
    @call do_ascii_to_decimal
    >[-]>[-]
]<<

subtract 3 more to check for d (total 100)
---
[>+>+<<-]>>[<<+>>-]<<
>>+<[>-<[-]]
>[
    -
    <<[-]
    @call do_decimal_to_ascii
    >[-]>[-]
]<<

subtract 8 more to check for l (total 108)
--------
[>+>+<<-]>>[<<+>>-]<<
>>+<[>-<[-]]
>[
    -
    <<[-]
    @call do_lower
    >[-]>[-]
]<<

subtract 9 more to check for u (total 117)
---------
[>+>+<<-]>>[<<+>>-]<<
>>+<[>-<[-]]
>[
    -
    <<[-]
    @call do_upper
    >[-]>[-]
]<<
