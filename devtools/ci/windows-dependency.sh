#!/bin/bash
set -ev

# powershell -Command "Set-ExecutionPolicy -ExecutionPolicy Bypass"
# powershell -Command "Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://get.scoop.sh')"
# set PATH=%PATH%;%USERPROFILE%\scoop\shims
# echo "##vso[task.setvariable variable=PATH;]%PATH%;%USERPROFILE%\scoop\shims"
# powershell -Command "scoop help"

powershell -Command "Set-ExecutionPolicy Bypass -scope CurrentUser";
powershell -Command "iex (new-object net.webclient).downloadstring('https://get.scoop.sh')";
export PATH="$HOME/scoop/shims:$PATH;"; 

powershell -Command "scoop install llvm"
powershell -Command "scoop install msys2"
# powershell -Command "scoop install yasm"
choco install yasm

rustc --version
cargo --version
clang --version
yasm --version

powershell -Command "cargo build --release"