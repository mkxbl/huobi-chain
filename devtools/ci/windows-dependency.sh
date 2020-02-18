#!/bin/bash
set -ev

powershell -Command "Set-ExecutionPolicy Unrestricted -Force"
powershell -Command "Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')"

scoop install llvm
scoop install msys2
scoop install yasm

rustc --version
cargo --version
clang --version
yasm --version

cargo build --release