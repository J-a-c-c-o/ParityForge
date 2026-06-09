mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers --tl --count 100 --size 100000