bin=./target/release/simple_mc
$bin $1 < tools/in/0000.txt > 0000.txt && \
    ./target/release/vis tools/in/0000.txt 0000.txt
