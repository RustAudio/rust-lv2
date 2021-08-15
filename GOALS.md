# Goals for the Rust `lv2` crate

The `lv2` crate aims to enable Rust developers to use [LV2](https://lv2plug.in/) to create plugins
(and later, hosts), by providing them with a set of safe, idiomatic, extensible, and powerful APIs.

This document details the reasoning for those goals, as well as a few more. These major goals directly
dictate our API design guidelines, as they tell which users we direct this library towards.

## Making a Safe library

Safety is at the core of Rust's language and API design. If you don't need safety (mainly, memory and thread safety,
among others), then using Rust probably doesn't make sense for you, making this library irrelevant to your needs.

This library follows Rust's [official definition of Safety and Unsafety](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html),
as detailed in the [Rustonomicon](https://doc.rust-lang.org/nomicon/).

In short, APIs provided by this library that are not marked `unsafe` cannot, under any circumstance, trigger Undefined
Behavior. This is even more important in the context of real-time processing: incoherent output, or the whole system
going down, can have terrible consequences.

This library, however, does not try to be resistant against incorrect LV2 host implementations. Aside from checking
pointers to be not-null, and checking array indexing to be in-bounds, there is little that a plugin can do. Even when
a plugin can detect it, there is little do to alert the user something is wrong, other than printing to stderr and
going silent.

Therefore, we need to fully and blindly trust that all the data given to the plugin from the host is correct.

One exception to this rule are the manifest files (the `.ttl` files distributed alongside the plugin binaries). They
are usually user-written, and writing them incorrectly is likely to trigger Undefined Behavior, in both the plugin and
the host.

Higher-level, more specialized libraries and frameworks that rely on LV2 should probably auto-generate those crates based on
plugin code and metadata. However, for the low-level control that `lv2` provides, these files need to be user-written
for now. It is possible that APIs relying on the content of those files could be marked `unsafe` in the future, even
if that makes it less practical to use for direct users of the `lv2` crates.

## No restrictions coming from C



## Idiomatic library
## Modularity and extensibility
## Lightweight, fast library
## Low-level library

## Extra ergonomics and sugar

## Host support

## `#![no_std]` support

