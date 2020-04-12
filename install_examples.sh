#!/usr/bin/env sh
cargo build --all --release

rm -rf target/lv2
mkdir -p target/lv2

cp -r docs/amp/eg-amp-rs.lv2 target/lv2/eg-amp-rs.lv2
cp target/release/libamp.so target/lv2/eg-amp-rs.lv2

cp -r docs/midigate/eg-midigate-rs.lv2 target/lv2/eg-midigate-rs.lv2
cp target/release/libmidigate.so target/lv2/eg-midigate-rs.lv2

cp -r docs/fifths/eg-fifths-rs.lv2 target/lv2/eg-fifths-rs.lv2
cp target/release/libfifths.so target/lv2/eg-fifths-rs.lv2

cp -r docs/metro/eg-metro-rs.lv2 target/lv2/eg-metro-rs.lv2
cp target/release/libmetro.so target/lv2/eg-metro-rs.lv2

mkdir -p ~/.lv2
cp -r target/lv2/* ~/.lv2