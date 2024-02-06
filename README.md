# ogre

## Run
```
(ogre run) <file>
```

- Simulates the file and prints the final output of the memory registers

## Debug
```
(ogre debug) <file>
```

- Starts a debugger interface for a file
- Stepping:
	- Stepping through each instruction
	- Stepping through each line (?)
	- Shows the code that is run after each "step" command
- Peek:
	- At a memory location (e.g. 10)
	- At a memory block (e.g. 10 - 20)

Example output:
```
$ (ogre debug) <file>

=====
Ogre Debugging: filename.bf
=====
> step       (steps 1 character)
=====
COMMAND: +
CURRENT MEMORY POINTER: 0
CHANGES: 
+ MEMORY VALUE 0 -> 1
MEMORY VALUES:
[0] [1] [2]
 1   0   0
=====
> step line      (steps 1 line)
=====
COMMAND: +>++>+++
CURRENT MEMORY POINTER: 2
CHANGES: 
+ MEMORY VALUE 0 -> 1
> MEMORY POINTER 0 -> 1
+ MEMORY VALUE 0 -> 1
+ MEMORY VALUE 1 -> 2
> MEMORY POINTER 1 -> 2
+ MEMORY VALUE 0 -> 1
+ MEMORY VALUE 1 -> 2
+ MEMORY VALUE 2 -> 3
MEMORY VALUES:
[0] [1] [2]
 1   2   3
=====
> step line 10   (steps 10 lines)
=====
COMMANDS:
[1]  +>++>++++
[2]  +++[>++<->]
...
CURRENT MEMORY POINTER: N

=====
> peek 1     (shows the value at location 1)
> peek 1 10  (shows the values at locations 1 to 10 inclusive)
> peek all   (shows all non-zero values)
> peek here  (shows current memory pointer location +-5 cells)
> save <output-filename> (saves log output to file)
> quit
> exit
```

## Start
```
ogre start
```
