#export RUST_LOG=trace
#export RUST_LOG=info
#export RUST_LOG=warn
export RUST_BACKTRACE=full

cargo run -- --cp test HelloWorld
