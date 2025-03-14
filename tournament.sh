#!/bin/bash

engines=""

#engines+=" -engine cmd=stockfish"
engines+=" -engine cmd=target/release/chess-master"

#for engine in binaries/*; do
#	engines+=" -engine cmd=$engine"
#done

engines+=" -engine cmd=binaries/final"

echo "$engines"

cutechess-cli $engines -tournament round-robin -games 500 -concurrency 25 -pgnout pgn -recover -each proto=uci st=0.01 timemargin=100
