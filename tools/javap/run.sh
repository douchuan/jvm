export RUST_LOG=trace
#export RUST_LOG=info
#export RUST_LOG=warn
export RUST_BACKTRACE=full

### version
#cargo run -- --version

### line number
#cargo run -- --cp test -l AbstractGraphicObject
#cargo run -- --cp test -l GraphicObject
#cargo run -- --cp test -l HelloWorld
#cargo run -- --cp test -l EnumMobile
#cargo run -- --cp test -l Interface1
#cargo run -- --cp test -l Hockey

### disassemble
#cargo run -- --cp test -c HelloWorld

### disassemble & line number
#cargo run -- --cp test -c -l HelloWorld
#cargo run -- --cp test -c -l EnumMobile


#cargo run -- --cp test --constants HelloWorld

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
