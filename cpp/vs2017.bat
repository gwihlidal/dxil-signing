@echo off
mkdir build
cd build
cmake ../. -G "Visual Studio 15 Win64"
cd ../