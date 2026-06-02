mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers -e "../target/release/ParityTool solve %I %O --algorithm zlk" -e "../target/release/ParityTool solve %I %O --algorithm fpi" -e "../target/release/ParityTool solve %I %O --algorithm tl" -e "../target/release/ParityTool solve %I %O --algorithm si" ../oink/tests