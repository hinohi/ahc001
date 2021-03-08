#!/bin/zsh

set -eu

n=$1
echo {0010..0017} |\
  tr ' ' '\n' | \
  xargs -P 4 -I@ sh -c "./target/release/simple_mc < tools/in2/$n/@.txt > out/@.txt"
for i in {0010..0017}; do
  ./target/release/vis "tools/in2/$n/$i.txt" "out/$i.txt"
done
