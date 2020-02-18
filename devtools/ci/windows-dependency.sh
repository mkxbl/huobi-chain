#!/bin/bash
set -ev

powershell -Command "Set-ExecutionPolicy -ExecutionPolicy Bypass"
powershell -Command "Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')"

powershell -Command "scoop install llvm"
powershell -Command "scoop install msys2"
powershell -Command "scoop install yasm"

rustc --version
cargo --version
clang --version
yasm --version

cargo build --release