Sun is a great company, in the era of the c++, they created JVM & HotSpot.
Now, we have Rust, a better tool, let’s see, whether we can remake JVM.

## Overview
 
The project is built with a month's holiday. It's a special holiday! 

Pay tribute to  frontline medical staff! Thank you for your contribution to the fight against the epidemic.

## Usage

Download JDK from
https://www.azul.com/downloads/zulu-community

Modify the run.sh script according to your environment

## Road map

- pass test cases in JDK 
- WebAssembly, make the JVM work in Browser 
- separate class parser from project as one standalone create
- support thread
- support GC, then optimize System.arraycopy
- split frame.rs into frame.rs & interp.rs
- pass TCK
- support higher version of JVM Spec 
