#!/bin/bash

engines=""

#engines+=" -engine cmd=stockfish"
engines+=" -engine cmd=target/release/chess-master"

#for engine in binaries/*; do
#	engines+=" -engine cmd=$engine"
#done

engines+=" -engine cmd=binaries/v4"

echo "$engines"

cutechess-cli $engines -tournament gauntlet -games 10 -concurrency 10 -pgnout pgn -each proto=uci st=0.1 timemargin=250 $@

