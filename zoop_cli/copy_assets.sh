#!/usr/bin/env bash

# Debug CLI
mkdir -p ../target/debug/
cp -rf ./assets/ ../target/debug/

# Release CLI
mkdir -p ../target/debug/
cp -rf ./assets/ ../target/release/

# Web
mkdir -p ../target/debug/
cp -rf ./assets/ ../target/debug/
