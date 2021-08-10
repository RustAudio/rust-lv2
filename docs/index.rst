Programming LV2 Plugins - Rust Edition
======================================

This is a series of well-documented example plugins that demonstrate the various features of `LV2`_ and how they translate to `rust-lv2`_. Starting with the most basic plugin possible, each adds new functionality and explains the features used from a high level perspective.

API and vocabulary reference documentation explains details, but not the "big picture". This book is intended to complement the reference documentation by providing good reference implementations of plugins, while also conveying a higher-level understanding of LV2.

The chapters/plugins are arranged so that each builds incrementally on its predecessor. Reading this book front to back is a good way to become familiar with modern LV2 programming. The reader is expected to be familiar with Rust, but otherwise no special knowledge is required; the first plugin describes the basics in detail.

Each chapter corresponds to executable plugin code which can be found in the  `project's doc folder`_. If you prefer to read actual source code, all the content here is also available in the source code as comments.

This book is a "translation" of `the original LV2 Book by David Robillard`_. Most of the examples as well as the README's and comments are copied from the original and have been altered to adapt for the differences between C and Rust.

.. toctree::
   :maxdepth: 2
   :caption: Chapters:

   chapter/amp
   chapter/midigate
   chapter/fifths
   chapter/metro

.. _LV2: https://lv2plug.in/
.. _rust-lv2: https://github.com/RustAudio/rust-lv2
.. _project's doc folder: https://github.com/RustAudio/rust-lv2/tree/master/docs
.. _the original LV2 Book by David Robillard: http://lv2plug.in/book/