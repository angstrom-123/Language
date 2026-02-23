# Language (Pending a Better Name)
My attempt at writing a programming language in Rust.
This language will support compilation to x86_64 assembly on Linux and it will 
support interpretation.

## Features
### Keywords
#### Debug Dump
Prints a number to the console for debugging.
```
dump <int>;
```

#### Return
Exits the program with a provided exit code.
```
return <int>;
```

### Arithmetic Operators
#### Addition
```
<int> + <int>
```

#### Subtraction
```
<int> - <int>
```

#### Multiplication
```
<int> * <int>
```

#### Integer Division
```
<int> / <int>
```
