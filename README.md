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

- pass test cases in JDK 
- WebAssembly, make the JVM work in Browser 
- separate class parser from project as one standalone create
- support thread
- support GC, then optimize System.arraycopy (the key of performance)
- split frame.rs into frame.rs & interp.rs
- pass TCK
- support higher version of JVM Spec 
- java options (-version, -server...)
