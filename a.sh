bin=./target/release/simple_mc
n=$1
$bin < tools/in/$n.txt > $n.txt && \
    ./target/release/vis tools/in/$n.txt $n.txt
rm -f $n.txt
