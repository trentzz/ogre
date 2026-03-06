# Standard Library Ideas

## New Modules

### data_structures.bf - Data Structure Primitives
Implement common data structures on the BF tape:
- **Stack**: push, pop, peek, is_empty (using a region of tape as a stack)
- **Queue**: enqueue, dequeue, front (circular buffer on tape)
- **Linked list**: insert, delete, traverse (pointer-chasing on tape cells)
- **Array utilities**: index_get, index_set, linear_search, bubble_sort
- **Bitmap**: set_bit, clear_bit, test_bit (pack 8 booleans into one cell)

### format.bf - Output Formatting
Higher-level formatting functions:
- **print_padded_left**: Right-align a number in a field width
- **print_padded_right**: Left-align a number in a field width
- **print_repeated**: Print a character N times (horizontal rules, padding)
- **print_table_row**: Format cells with separators
- **print_centered**: Center text in a given width
- **print_hex_byte**: Print two-digit hex value (00-FF)
- **print_octal**: Print value in octal
- **print_boolean**: Print "true" or "false"

### crypto.bf - Novelty Cryptography
Simple ciphers implementable in BF (educational, not secure):
- **caesar_encrypt / caesar_decrypt**: Shift cipher
- **rot13**: Classic ROT13
- **xor_encrypt**: XOR with a key byte
- **checksum**: Simple additive checksum
- **hash_djb2**: DJB2 hash function (as much as possible in 8-bit cells)

### game.bf - Game Utilities
Primitives for interactive BF programs:
- **read_arrow_key**: Parse arrow key escape sequences
- **clear_screen**: Print ANSI clear screen sequence
- **move_cursor**: ANSI cursor positioning
- **set_color**: ANSI color codes (foreground/background)
- **reset_color**: Reset to default terminal colors
- **print_box**: Draw a box with unicode/ASCII box-drawing characters
- **delay_loop**: Waste N instructions (crude timing)

### validate.bf - Input Validation
Validate user input:
- **is_yes_no**: Check if input is y/n/Y/N
- **is_numeric_string**: Check if all chars are digits
- **is_hex_string**: Check if all chars are hex digits
- **is_alpha_string**: Check if all chars are letters
- **read_validated_int**: Read and validate a decimal number
- **clamp**: Constrain a value to a range

---

## Enhancements to Existing Modules

### math.bf Additions
- **divide**: Integer division (a / b)
- **power**: Exponentiation (a^b)
- **factorial**: n! (limited to small values due to 8-bit cells)
- **gcd**: Greatest common divisor (Euclidean algorithm)
- **lcm**: Least common multiple
- **sign**: Return 1 for positive, 0 for zero (already is_nonzero but semantically different)
- **add_const / sub_const**: Add/subtract a compile-time constant more efficiently than N increments
- **fibonacci_nth**: Compute the Nth Fibonacci number

### string.bf Additions
- **to_upper_string**: Convert entire string to uppercase
- **to_lower_string**: Convert entire string to lowercase
- **reverse_string**: Reverse a null-terminated string in place
- **trim_spaces**: Remove leading/trailing spaces
- **count_char**: Count occurrences of a character in a string
- **find_char**: Find position of first occurrence
- **concat**: Concatenate two strings
- **substring**: Extract substring by position and length
- **starts_with**: Check if string starts with a given character
- **ends_with**: Check if string ends with a given character

### memory.bf Additions
- **memcpy**: Copy N cells from one region to another
- **memset**: Fill N cells with a given value
- **memcmp**: Compare N cells between two regions
- **memmove**: Safe copy even when regions overlap
- **find_zero**: Scan forward to find next zero cell
- **count_nonzero**: Count non-zero cells in a range
- **checksum_range**: Compute additive checksum of a cell range

### io.bf Additions
- **print_string_literal**: Print a compile-time string (could use @macro)
- **read_line_buffered**: Read a line into a cell range, return length
- **print_int**: Alias for convert.print_decimal (convenience)
- **read_int**: Alias for convert.atoi (convenience)
- **println**: Print newline after current cell's character
- **print_n_newlines**: Print N blank lines

### logic.bf Additions
- **ternary**: If cell[0] then cell[1] else cell[2] -> cell[0]
- **min / max**: Already in math, but expose via logic for clarity
- **all_zero**: Check if N consecutive cells are all zero
- **any_nonzero**: Check if any of N cells is nonzero
- **bit_and / bit_or / bit_xor**: Bitwise operations (using the 8-bit cell)
- **bit_shift_left / bit_shift_right**: Shift bits within a cell

### ascii.bf Additions
- **is_whitespace**: Space, tab, newline, carriage return
- **is_punctuation**: Check if char is punctuation
- **is_alphanumeric**: Letter or digit
- **is_vowel**: Check if char is a vowel (a, e, i, o, u)
- **is_consonant**: Letter but not a vowel
- **to_digit**: Convert '0'-'9' to 0-9 (alias for char_to_digit)

### convert.bf Additions
- **print_hex_byte**: Print two-digit hex (e.g., cell value 255 -> "FF")
- **parse_hex_byte**: Parse two hex chars to a cell value
- **print_octal**: Print value in octal
- **print_binary_4**: Print lower 4 bits only
- **celsius_to_fahrenheit**: Fun novelty conversion
- **ascii_table_entry**: Print the ASCII table entry for a cell value
