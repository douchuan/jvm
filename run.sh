CLASS_PATH=.:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar

###
#cargo run -- --cp $CLASS_PATH test/Add
#cargo run -- --cp $CLASS_PATH test/HelloWorld 123 456 789
#cargo run -- --cp $CLASS_PATH test/Ex

### Overflow
#cargo run -- --cp $CLASS_PATH test/SubOverflow

### Enum CloneNotSupportedException
#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp $CLASS_PATH test/EnumDemo
#cargo run -- --cp $CLASS_PATH test/EnumDemo

### jdk test
#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp $CLASS_PATH test/Compare
#cargo run -- --cp $CLASS_PATH test/Compare
