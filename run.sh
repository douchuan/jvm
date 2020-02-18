
###test/HelloWorld 123, with rust RUST_LOG
#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/HelloWorld 123

###test/HelloWorld 123, without RUST_LOG
#cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/HelloWorld 123

#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/Ex
#cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/Ex

#cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/SubOverflow

#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/Compare
cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar test/Compare

#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar test/HelloWorld
#RUST_LOG=trace RUST_BACKTRACE=1 cargo run -- --cp .:test/:test/zulu8/jre/lib/rt.jar test/Add
