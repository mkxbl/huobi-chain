#!/bin/bash

echo "figure out where i am"
echo $PWD

mkdir -p packages/$TRAVIS_OS_NAME
cp -r ./config ./packages/$TRAVIS_OS_NAME/
cp ./target/release/huobi-chain ./packages/$TRAVIS_OS_NAME/
tar zcvf huobi-chain-$TRAVIS_OS_NAME.tar.gz packages/$TRAVIS_OS_NAME/