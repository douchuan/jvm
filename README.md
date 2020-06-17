Have a great goal is a very important thing, such as Moon Landing.
What is the meaning of this plan? 
That's the process of achieving this goal, the produce of industry and technology.

Sun is a great company, in the era of C++, they created JVM & HotSpot.

Now, we have Rust, a better tool, let’s remake JVM! 

Pay tribute to medical workers at the front! Thank you for your contribution to the fight against the epidemic.


## Roadmap

- Pass test cases in JDK 
- Pass TCK 
- GC (crate)
- JIT / interp (crate)
- class verification (crate)
- After GC built, ready for optimize System.arraycopy (the key of performance)
- WebAssembly, make the JVM work in Browser 
- java options (-version, -server...)

In summary, the roadmap is built on a 3-step progress.
- Pass TCK
- Refactor & Rewrite
- Divide into several crates, build a collection of modular and reusable vm technologies

Well, it's a long term plan, Sun spent 30 years to improve
VM, Oracle continue doing it.

The journey of a thousand miles begins with one first step. Even the sage was once an ordinary human being.

Just Do It.

## Running
```shell
# setup JDK
# setup rust toolchain
# clone this project

# compile sample
cd jvm/sample
javac HelloWorld.java
cd ..

# exec sample
cd jvm
sh r.sh
```