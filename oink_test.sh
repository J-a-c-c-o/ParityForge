echo "Zylonka alogrithm test"
./oink/build/test_solvers -e "./target/release/ParityTool solve %I %O --algorithm zielonka" -e "./target/release/ParityTool solve %I %O --algorithm fpi" -e "./target/release/ParityTool solve %I %O --algorithm tl" ./oink/examples/
