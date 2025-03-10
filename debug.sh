#!/bin/sh

tee /home/user/stdin.txt | /home/user/chess-master/target/release/chess-master | tee /home/user/stdout.txt
