Have a great goal is a very important thing, such as Moon Landing.
What is the meaning of this plan? 
That's the process of achieving this goal, the produce of industry and technology.

Sun is a great company, in the era of C++, they created JVM & HotSpot.

Now, we have Rust, a better tool, let’s remake JVM! 

Pay tribute to  frontline medical staff! Thank you for your contribution to the fight against the epidemic.

## Usage

Modify the run.sh script according to your environment.

If you installed JDK, but can not find jre path, try:

```shell
java -XshowSettings:properties
```

## Roadmap

- Pass test cases in JDK (TCK) 
- Separate class parser from project as one standalone crate
- After class parser crate is finished, we can write javap
- Support threads
- WebAssembly, make the JVM work in Browser 
- Support GC
- After GC built, ready for optimize System.arraycopy (the key of performance)
- Support higher version of JVM Spec 
- java options (-version, -server...)
- Split frame.rs into frame.rs & interp.rs

In summary, separated into three steps
- Pass all test cases, to verify implementation is ok
- Based on the first phase of the results, refactor & rewrite
- Split the project to crates, build a collection of modular and reusable vm technologies

Well, it's a long roadmap, Sun spent 30 years to improve
vm, Oracle continue doing it.

The journey of a thousand miles begins with one first step. Even the sage was once an ordinary human being.

Just do it.