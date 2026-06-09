mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers --tl ../pg_files/*.pg