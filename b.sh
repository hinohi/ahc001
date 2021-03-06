#!/bin/zsh

echo {0000..0048} |\
  tr ' ' '\n' | \
  xargs -P 4 -I@ sh -c './target/release/simple_mc < tools/in/@.txt > out/@.txt'
for n in {0000..0048}; do
  ./target/release/vis "tools/in/$n.txt" "out/$n.txt"
done
