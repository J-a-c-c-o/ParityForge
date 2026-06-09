mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers -e '../target/release/ParityForge solve --algo tl %I %O' ../oink/tests/vb158 -t 3