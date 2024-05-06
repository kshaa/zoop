#!/usr/bin/env bash

# Dev CLI
rm -rf ../zoop_cli/assets
cp -rf ./assets/ ../zoop_cli/

# Debug CLI
mkdir -p ../target/debug/
rm -rf ../target/debug/assets
cp -rf ./assets/ ../target/debug/

# Release CLI
mkdir -p ../target/release/
rm -rf ../target/release/assets
cp -rf ./assets/ ../target/release/

# Web
rm -rf ../zoop_web/public/assets
cp -rf ./assets/ ../zoop_web/public/
