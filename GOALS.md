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

Safe APIs that can trigger Undefined Behavior are *unsound*, and we consider these bugs: they are either implementation
bugs or full API design flaws. These are marked with the `Unsound API` tag on GitHub, for better visibility.

**Note:** *While this library is in an alpha (`0.x`) stage, we might allow some soundness issues to stay for a bit, as
long as they are odd edge-case and not easy to trigger unwillingly. This is because the APIs are not complete
and still subject to change, however we will have to resolve all of these before getting to `1.0`.*

However, this library can provide APIs that are marked `unsafe`, if there are performance considerations in mind, or if
those are building blocks for higher levels of abstractions (such as Port Types). In such case, the whole API can be
made `unsafe`, if appropriate.

The main goal is for end-users to have to write as little `unsafe` code as possible by using this library, ideally none.

This library, however, does not try to be resilient against incorrect LV2 host implementations. Aside from checking
pointers to be not-null, and checking array indexing to be in-bounds, there is little that a plugin can do. Even when
a plugin can detect it, there is little it can do to alert the user something is wrong, other than printing to stderr
and going silent.

Therefore, we need to fully and blindly trust that all the data given to the plugin from the host is correct.

One exception to this rule are the manifest files (the `.ttl` files distributed alongside the plugin binaries). They
are usually user-written, and writing them incorrectly is likely to trigger Undefined Behavior, in both the plugin and
the host.

Higher-level, more specialized libraries and frameworks that rely on LV2 should probably auto-generate those crates based on
plugin code and metadata. However, for the low-level control that `lv2` provides, these files need to be user-written
for now. It is possible that APIs relying on the content of those files could be marked `unsafe` in the future, even
if that makes it less practical to use for direct users of the `lv2` crates.

## No restrictions coming from C

While LV2 is most used to make digital synthesizers and MIDI or Audio processing plugins, it can do a lot of other
things.

LV2 can do [weird](https://lv2plug.in/ns/ext/morph) things. Very, [very weird](https://lv2plug.in/ns/ext/dynmanifest) things.

As long as they can be implemented safely, this library's goal is to expose every single possibility the LV2 specifications
provide. (The `lv2` crate can implement a few unsafe APIs for better, lower-level control, but if you need full-unsafe,
C-low-level control, the `lv2-sys` crate can always be used directly, but only for extreme edge-cases.)

The core idea is to make sure that no user of the `lv2` crate could stumble upon a case where an API is too restrictive
compared to the official LV2 spec.

See also the next section about this being a low-level library.

**Note:** *This library is in an alpha (`0.x`) stage right now. While the goal for `1.0` is to have every official
specification fully implemented, there might be a lot of missing specs or functionalities until then.*

## Low-level library

We want to expose as many LV2 APIs as possible while giving as much control as possible to the user. This may expose 
tricky low-level details that DSP and UI developers shouldn't need to worry about (such as URIDs or Atoms).

However, because the `lv2` crate provides lots of flexibility for the user, it is easy to make more restrictive,
but easier to use high-level abstractions on top of this crate. Or at least, it is easier than trying to poke holes
through a high-level abstraction for some users that may need extra flexibility. Instead, those users can ditch the
abstractions (or parts of it) and use the low-level `lv2` library.

Having a single, well-integrated low-level library also helps to lay down a solid foundation for LV2 in the ecosystem.
This allows all the complexity (and unsafety) of LV2 internals to be abstracted and shared among all Rust LV2 users.

## Idiomatic library
## Modularity and extensibility
## Lightweight, fast library

## Extra ergonomics and sugar
Also, while ergonomics and extra utilities are nice to have in this library sometimes, they *must* be optional to use.
Indeed, because of the goal to be a low-level library, we must not prevent the user from doing custom things themselves
at the cost of complexity. At least, as long as it is safe for them to do so.

A good example of this is the `UridCollection` derive macro in the `urid` crate. While users can make similar
collections manually using 100% safe and sound code, it is a very tedious and boilerplate-heavy implementation that
can be abstracted away. This is thanks to the fact that `URID<T>` is integrated to the type system, allowing the
structure to be manually filled if desired.

An anti-example of this is the `PortCollection` derive macro in the `core` crate. It may seem similar to
`UridCollection` at a first glance (it is a collection of things that can be filled automatically), but there is a
catch. An invalid `index <-> port` mapping implementation means that bad pointer casts are going to be made, and this
is definitely Undefined Behavior.

In this case, this ergonomic helper is necessary to produce safe code, and is not considered optional.

## Host support

## `#![no_std]` support

