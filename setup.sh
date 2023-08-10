#!/bin/bash

cd ./BarFlowController
cargo build --release
docker build -t barflowcontroller .
cd ..
docker-compose up -d
