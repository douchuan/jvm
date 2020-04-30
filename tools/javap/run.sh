export RUST_LOG=trace
#export RUST_LOG=info
#export RUST_LOG=warn
export RUST_BACKTRACE=full

### version
#cargo run -q -- --version

### line number
#cargo run -q -- --cp test -l AbstractGraphicObject
#cargo run -q -- --cp test -l GraphicObject
#cargo run -q -- --cp test -l HelloWorld
#cargo run -q -- --cp test -l EnumMobile
#cargo run -q -- --cp test -l Interface1
#cargo run -q -- --cp test -l Hockey
#cargo run -q -- --cp test -l WifeAndMother

### disassemble
#cargo run -q -- --cp test -c HelloWorld

### disassemble & line number
#cargo run -q -- --cp test -c -l HelloWorld
#cargo run -q -- --cp test -c -l EnumMobile


#cargo run -q -- --cp test --constants HelloWorld
#cargo run -q -- --cp test --constants Hockey

### access flags
#echo "default access flags"
#cargo run -q -- --cp test HelloWorld
#echo "public"
#cargo run -q -- --cp test --public HelloWorld
#echo "protected"
#cargo run -q -- --cp test --protected HelloWorld
#echo "private"
#cargo run -q -- --cp test --private HelloWorld

### conflict args, error
#cargo run -- --cp test --private --public HelloWorld

### sysinfo
#cargo run -q -- --cp test --sysinfo HelloWorld
#cargo run -q -- --cp test/testng-6.8.21.jar --sysinfo org.testng.collections.Lists
#cargo run -q -- --cp test HelloWorld

### Print internal type signatures
#cargo run -q -- --cp test -s  HelloWorld
### exception
#cargo run -q -- --cp test  Ex
#cargo run -q -- --cp test -s Ex
#cargo run -q -- --cp test -c Ex
#cargo run -q -- --cp test -v Ex

#cargo run -q -- --cp test -v HelloWorld
#cargo run -q -- --cp test/testng-6.8.21.jar -v org.testng.xml.XmlUtils
#cargo run -q -- --cp test/testng-6.8.21.jar -v org.testng.xml.XmlUtils
#cargo run -q -- --cp test/testng-6.8.21.jar -v org.testng.TestNG

### contains generic params
#cargo run -q -- --cp test/testng-6.8.21.jar  -v org.testng.TestNG
#cargo run -q -- --cp test/testng-6.8.21.jar  -s org.testng.TestNG
#cargo run -q -- --cp test/testng-6.8.21.jar  -v org.testng.TestRunner
#cargo run -q -- --cp test/testng-6.8.21.jar  -v org.testng.reporters.Files
#cargo run -q -- --cp test/testng-6.8.21.jar  -v org.testng.collections.Maps
cargo run -q -- --cp test  -v Generic1


### test Not Found
#cargo run -q -- --cp test/testng-6.8.21.jar  -v passed.png
