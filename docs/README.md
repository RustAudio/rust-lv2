# Programming LV2 Plugins - Rust Edition

[![Build Status](https://travis-ci.org/Janonard/rust-lv2-book.svg?branch=master)](https://travis-ci.org/Janonard/rust-lv2-book)

This repository contains the sample plugins of the "Programming LV2 Plugins - Rust edition" book, as well as means to build both the plugins and the book.

## Building the book

The book is generated from the source files of the samples. In order to build the book, you need to have Python 3 installed. Simply type

```bash
python3 make_book.py
```

and the book will be written to `export/README.md`.

## Building and installing the sample plugins

Every sample is a self-contained Rust crate; You can simply build it with cargo. If you want to install the samples on your machine, you can run `install_examples.sh`. This will build the crates, bundle them and copy them to `~/.lv2`.

The compiler might complain that "profiles for the non root package will be ignored", which you can safely ignore. Some examples have a profile section to show how to enable link-time optimizations, but these profile section don't have an effect.

## Licensing

Just like the original, the book and the code is published under the `ISC` license. See the [LICENSE file](LICENSE.md) for more info.