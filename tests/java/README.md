# Java Test Suite

This directory contains Java source files to verify the JVM's capabilities. Each file targets a specific feature set.

## Quick Start

```bash
# Compile and run all tests
./tests/java/run.sh

# Run a single test
cargo run -p jvm -- --cp tests/java/out HelloWorld
```

## Test Files

| File | Category | Covers |
|------|----------|--------|
| `HelloWorld.java` | Basic | `main` entry, `System.out.println` |
| `SimpleCalc.java` | Basic | Constructor, instance method calls, arithmetic |
| `AllTypes.java` | Types | 8 primitive types, arrays as fields |
| `Interfaces.java` | OOP | Interface implementation, interface method dispatch |
| `Arithmetic.java` | Types | int/long/float/double ops, bitwise, type conversion |
| `ControlFlow.java` | Control | if/else, for, while, do-while, switch, break/continue |
| `Arrays.java` | Arrays | newarray, anewarray, multidim, bounds check, catch |
| `Exceptions.java` | Exception | try-catch-finally, throw, multiple catch, nested |
| `OopInheritance.java` | OOP | extends, super(), method override, field access |
| `OopPolymorphism.java` | OOP | invokevirtual, interface method dispatch |
| `OopEncapsulation.java` | OOP | getfield/putfield, private fields, encapsulation |
| `Strings.java` | Strings | concatenation, equals, length, charAt, substring, StringBuilder |
| `StaticInit.java` | Class Load | clinit, static blocks, class variables |
| `Generics.java` | Generics | ArrayList\<T\>, generic methods (type erasure) |
| `EnumDemo.java` | Class Load | enum, values(), ordinal(), name() |
| `Recursion.java` | Call Stack | recursive fib, factorial, binary search |
| `GCDemo.java` | GC | bulk allocation, OOM handling |

## Status

**17/17 passing.** All tests compile and run to completion.

## Build Notes

- `.class` files are not committed. Only `.java` sources are in version control.
- All files compile with standard `javac` (JDK 8+ compatible).
- The `run.sh` script compiles on-the-fly and tests each class with a 30s timeout.
