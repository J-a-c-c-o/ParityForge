mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers -e "../target/release/parityforge solve --algo fpj %I %O" -e "../target/release/parityforge solve --algo fpi %I %O" ../oink/tests/