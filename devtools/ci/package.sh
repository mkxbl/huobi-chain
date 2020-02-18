#!/bin/bash
set -ev

echo "figure out where i am"
echo $PWD
mkdir package
cp -r ./config ./package/
if [ "$TRAVIS_OS_NAME" == "windows" ]
then
    cp ./target/release/huobi-chain.exe ./package/
    7z a huobi-chain-$TRAVIS_OS_NAME-$TRAVIS_TAG.zip package/
else
    cp ./target/release/huobi-chain ./package/
    tar zcvf huobi-chain-$TRAVIS_OS_NAME-$TRAVIS_TAG.tar.gz package/
fi
