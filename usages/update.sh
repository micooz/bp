#!/bin/bash

DIR=`dirname $0`

cargo run -- -h > ${DIR}/bp.txt
cargo run client -h > ${DIR}/bp-client.txt
cargo run server -h > ${DIR}/bp-server.txt
cargo run generate -h > ${DIR}/bp-generate.txt
cargo run test -h > ${DIR}/bp-test.txt
cargo run web -h > ${DIR}/bp-web.txt
