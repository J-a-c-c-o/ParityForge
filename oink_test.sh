mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers --tl --zlk --fpi --psi ../oink/examples/