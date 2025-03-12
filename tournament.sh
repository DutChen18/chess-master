#!/bin/bash

#engines=""

#for engine in binaries/*; do
#	engines+=" -engine cmd=$engine"
#done

cutechess-cli -engine cmd=binaries/v3 -engine cmd=./target/release/chess-master -each proto=uci st=0.1 timemargin=1000 -tournament round-robin -pgnout pgn $@

