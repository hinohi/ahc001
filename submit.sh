#!/bin/bash

set -eu

BIN_NAME=a
SUBMIT_FILE=main.sh

function make_zip() {
  tar xf bin.tar $BIN_NAME
  strip $BIN_NAME
  gzip -9 $BIN_NAME
  hash=$(sha1sum $BIN_NAME.gz | sed -E 's/^(.{10}).*/\1/')
  base64 --break=100 -i $BIN_NAME.gz -o $BIN_NAME.gz.base64
  echo "cat << @ > /tmp/$hash.gz.base64"
  cat $BIN_NAME.gz.base64
  echo "@"
  echo "base64 -d -i /tmp/$hash.gz.base64 -o /tmp/$hash.gz"
  echo "gunzip /tmp/$hash.gz"
  echo "chmod u+x /tmp/$hash"
  echo "/tmp/$hash"
}

rm -rf dest
mkdir dest
docker build . -o type=tar,dest=dest/bin.tar
pushd dest
  make_zip > $SUBMIT_FILE
popd
