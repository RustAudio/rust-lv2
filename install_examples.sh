#!/usr/bin/env sh
set -e -x
[[ ! -z $TARGET ]] || (export TARGET="x86_64-unknown-linux-gnu")

rm -rf target/lv2
mkdir -p target/lv2

cargo build -p amp --release --target $TARGET
cp -r docs/amp/eg-amp-rs.lv2 target/lv2/eg-amp-rs.lv2
cp target/$TARGET/release/libamp.so target/lv2/eg-amp-rs.lv2

cargo build -p midigate --release --target $TARGET
cp -r docs/midigate/eg-midigate-rs.lv2 target/lv2/eg-midigate-rs.lv2
cp target/$TARGET/release/libmidigate.so target/lv2/eg-midigate-rs.lv2

cargo build -p fifths --release --target $TARGET
cp -r docs/fifths/eg-fifths-rs.lv2 target/lv2/eg-fifths-rs.lv2
cp target/$TARGET/release/libfifths.so target/lv2/eg-fifths-rs.lv2

cargo build -p metro --release --target $TARGET
cp -r docs/metro/eg-metro-rs.lv2 target/lv2/eg-metro-rs.lv2
cp target/$TARGET/release/libmetro.so target/lv2/eg-metro-rs.lv2

mkdir -p ~/.lv2
cp -r target/lv2/* ~/.lv2