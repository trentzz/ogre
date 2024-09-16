"""
(ogre analyse) : analyses a brainfuck script, giving details about
    - where the pointer starts and ends
    - how many total inputs/outputs there are

this can also be done for sections of code, e.g.

code code code

=== <-- this is a section marker

code code code

example output:

=====================
SECTION 1
=====================
lines:          1 - 100
ptr start:      0    
ptr end:        12
mem changed:    0-12
=====================
SECTION 2
=====================
...



options:

--in-place  analysis goes straight into code, no line numbers added
--verbose   extra information
"""