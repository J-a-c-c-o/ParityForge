echo "Zylonka alogrithm test"
./oink/build/test_solvers -e "./target/release/ParityTool solve %I %O" | tail -n 3

echo ""

echo "FPI algorithm test"
./oink/build/test_solvers -e "./target/release/ParityTool solve %I %O --algorithm fpi" | tail -n 3


