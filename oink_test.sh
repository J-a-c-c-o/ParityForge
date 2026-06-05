mkdir -p tmp &&
cargo build --release && 
cd ./tmp && 
../oink/build/test_solvers -e "../target/release/ParityTool solve %I %O --algo zlk" -e "../target/release/ParityTool solve %I %O --algo tl"  -e "../target/release/ParityTool solve %I %O --algo fpi"  -e "../target/release/ParityTool solve %I %O --algo si" ../oink/examples/*.pg