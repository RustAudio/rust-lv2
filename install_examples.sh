#!/usr/bin/env sh
#
# Installation script for the example plugins.
#
# This script builds every plugin and installs it into `~/.lv2`, which is the standard path for user
# plugins for Linux. The first argument of the script is interpreted as the target to build the
# plugins for. If you leave it out, the default target of your Rust installation is used.
export TARGET_PATH="target"
test ! -z "$1" && export TARGET_OPT="--target $1" TARGET_PATH="target/$1"

set -e -x

rm -rf target/lv2
mkdir -p target/lv2

cargo build -p amp --release $TARGET_OPT
cp -r docs/amp/eg-amp-rs.lv2 target/lv2/eg-amp-rs.lv2
cp $TARGET_PATH/release/libamp.so target/lv2/eg-amp-rs.lv2

cargo build -p midigate --release $TARGET_OPT
cp -r docs/midigate/eg-midigate-rs.lv2 target/lv2/eg-midigate-rs.lv2
cp $TARGET_PATH/release/libmidigate.so target/lv2/eg-midigate-rs.lv2

cargo build -p fifths --release $TARGET_OPT
cp -r docs/fifths/eg-fifths-rs.lv2 target/lv2/eg-fifths-rs.lv2
cp $TARGET_PATH/release/libfifths.so target/lv2/eg-fifths-rs.lv2

cargo build -p metro --release $TARGET_OPT
cp -r docs/metro/eg-metro-rs.lv2 target/lv2/eg-metro-rs.lv2
cp $TARGET_PATH/release/libmetro.so target/lv2/eg-metro-rs.lv2

mkdir -p ~/.lv2
cp -r target/lv2/* ~/.lv2
