# Language (Pending a Better Name)
My attempt at writing a compiled programming language in Rust for x86_64 Linux.
This language compiles to assembly, assembles with nasm, and 
links with ld into a native executable.

## Usage
### Compile the Program
```
cargo run <file_path> <flags>
```
> [!NOTE}
> Currently, compiled files are placed in the project root and named "output".

| Flag         | Shorthand | Meaning               |
| ------------ | --------- | --------------------- |
| --parse-tree | -pt       | Print parse tree      |
| --assembly   | -a        | Keep intermediate asm |
| --tokens     | -t        | Print lexed tokens    |
| --run        | -r        | Run after compiling   |

## Examples
An examples folder is included with the project showcasing the language features 
and giving real syntax examples. Combined with the listed features below, this 
should be enough to get a basic grasp on the syntax.

## Language Features
### Keywords
#### Debug Dump
Prints a number to the console for debugging.
```
dump <int>;
```

#### Exit
Exits the program with a provided exit code.
```
exit <int>;
```

#### Function Declaration
Declares a new named funcion.
```
func <function_name> {
    <body>
}
```

#### Function Call
Calls a previously declared funcion.
```
<function_name>();
```
