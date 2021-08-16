# Goals for the Rust `lv2` crate

The `lv2` crate aims to enable Rust developers to use [LV2](https://lv2plug.in/) to create plugins
(and later, hosts), by providing them with a set of safe, idiomatic, extensible, and powerful APIs.

This document details the reasoning for those goals, as well as a few more. These major goals directly
dictate our [API design guidelines](./DESIGN.md), as they tell which users we direct this library towards.

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

LV2 can do [weird](https://lv2plug.in/ns/ext/morph) things. [Very weird](https://lv2plug.in/ns/ext/dynmanifest) things.

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

This means that this library isn't ultimately designed to be the best possible development experience for plugin
authors. While plugins can be written directly on top of the `lv2` crate, users should expect having to handle
low-level details that can come in their way. There are also many, sometimes complex, abstractions that this crate
exposes, whose main purpose is to hide the pointer-juggly type twisting LV2 requires doing.

However, we do really appreciate plugin authors that choose to help to battle-test this crate by writing plugins on
top of it! Thank you very much! <3

## Idiomatic library

Just because we are writing a low-level wrapper for a complex C API, doesn't mean we can't expose a nice, idiomatic
Rust library when we can.

The [API Design Guidelines](./DESIGN.md) covers this in more detail, but the main goal is to integrate as closely
as possible with the Rust standard library, using types and traits such as `Result`, `Iterator`, `Debug`, `Error`, and
many more.

We also want to provide a good integration with other crates from the Rust ecosystem, as long as they are locked behind
optional, non-default [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html). This way the user can
opt-in on better integration with the (sub-)ecosystem of their choosing, while keeping a minimal dependency tree.

Examples of crates that can be good to integrate with are `serde`, `wmidi`, or `baseview`, but there can be many more.

## Modularity and extensibility

The LV2 API is, by design, modular and extensible: only the minimal [LV2 Core specification]() is actually
required to make a working LV2 plugin. Everything else is a separate specification (and a separate header file) that
is built on top of it.

The `lv2` crate enforces this by having every single LV2 specification implemented in a separate sub-crate. In fact, 
the `lv2` crate itself is nothing but re-exports of the sub-crates, each in a separate module gated by
[Cargo features](https://doc.rust-lang.org/cargo/reference/features.html). The `lv2` crate, in itself, is designed to be
nothing but a nice landing point for users.

For instance, the [LV2 MIDI specification](http://lv2plug.in/ns/ext/midi) is implemented in the `lv2-midi` crate, and is
re-exported in the `lv2` crate under the `midi` submodule (gated by the `midi` feature). 
This way, users can choose to either depend on the `lv2` crate, or on the specific sub-crates they need.

Because we implement specifications as separate crates, we can make sure that there are no private implementation
details shared across specifications, preventing users to implement their own if needed. 
This has several big advantages: 

* Users can always pick and choose what they need, and not include what they don't.
  
  Although this is unlikely to impact runtime performance, it does help to reduce the amount of dependencies,
  as well as compile times and final binary sizes, which is always nice.
* Users can swap out some specification implementations for their own, while still relying on the rest of the `lv2`
  crate(s).

  Although the goal of this library is to cover as many use cases as possible, it may be possible for some users to
  stumble upon extreme edge-cases we didn't see coming. 
* With this library still being in alpha (`0.x`) state, some specifications might be incomplete, or not implemented
  at all. This allows the user to put together a quick implementation that suits their needs while they wait for the
  full specification to be implemented. (Pull Requests are always welcome however!)
* Users can implement (and publish) non-standard LV2 specifications on top of the `lv2` crates. This is by far the
  biggest advantage, as the LV2 ecosystem also uses non-standard but useful specifications. For instance, the
  [KxStudio](https://kx.studio) project has a few [extra specifications](https://kx.studio/ns/) that some plugins
  implement. The [Ardour](https://ardour.org) DAW also has some non-standard specifications for their inline strip
  displays, which can be quite useful.

## Lightweight, fast library

Obviously, any library handling realtime audio needs to be fast to stay within the time budgets of the given audio
buffer sizes (and avoid XRuns).

This is where Rust's "zero-cost abstractions" truly shine, as we can build higher-level abstractions that produce little
to no extra machine code to execute (and thus a minimal performance penalty). Of course, the `lv2` crate provides a
large majority of those in its APIs, a vast amount being nothing but a wrapper around the pointers given to the plugin
by the host.

However, there is another important performance consideration that is not just "how fast can we do the processing":
plugin implementations need to be as lightweight as possible. This means than both plugin authors, and the libraries
they use (such as the `lv2`), must be *extremely* conservative about the resources they allocate for themselves.
Whether it is memory, threads, or other synchronization primitives that can introduce delays or locking.

The reason for this is simple: a plugin is very rarely alone when it's used in a DAW. It is most likely to run alongside
dozens, if not hundreds, of other plugins. Not to mention the processing the DAW itself needs to do for mixing all of
these, and the I/O it needs to perform to the hardware to get actual sound. All of it in very tight timing budgets.

For instance, audio processing code may want to spread work across multiple threads to be as fast as possible. This
works fine in a standalone application where the process can allocate most or all of the CPU cores to itself. However,
this can't work in a session where there are many instances of that code competing for CPU power at the same time. The
OS would have to interrupt and reschedule threads constantly, losing most of the CPU power in context switches.

In general, the LV2 APIs consider the host to be in charge of handling most of the plugin's resources, and behave
like a scheduler or executor of sorts for the various plugins. Because it knows the state of all the plugins running,
as well as their (potentially complex) I/O configuration, it can apply massive optimizations to its scheduling and
processing, such as spreading the work on a single thread pool. However, this means the plugins must cooperate, and
cannot do what they want. Examples of work LV2 plugins should defer to the host are I/O buffers and communication, state
serialization (i.e., presets / session loading and saving), asynchronous processing, or UI communication.

In that aspect, LV2 plugins and hosts share very similar behaviors with Rust's own
[Futures](https://doc.rust-lang.org/std/future/trait.Future.html) and
[Executors](https://docs.rs/futures/0.3.16/futures/executor/index.html).

Like with Rust Futures, the execution system for LV2 plugins is inherently *cooperative*. LV2 plugins *must* finish
processing before another plugin can run. If a plugin, just like a Future, uses blocking operations, it will block
the whole thread without any means of interruption by the host. This includes (but is not limited to): memory
allocations, multi-thread processing, thread synchronization (atomics, locks), general I/O, and more.

If a plugin needs to perform asynchronous work for instance (like loading and decoding a sample file), they should use
the [LV2 Worker](http://lv2plug.in/ns/ext/worker) API (see the `lv2-worker` crate). Just like a Future would use the
executor's [`spawn()`](https://docs.rs/futures/0.3.16/futures/task/trait.SpawnExt.html#method.spawn) method to process
additional asynchronous work, instead of doing it synchronously or spawning a thread of its own.

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

In this case, this ergonomic helper is necessary to produce safe code, and is not considered optional: the
`PortCollection` abstraction cannot be bypassed.

## Host support

This library is intended to provide host support at some point. However, considering our currently limited resources,
we took the decision to first get sufficient support for implementing plugins, before focusing on host support.

However, host support will need to be (or at least, guaranteed to not have backwards-compatibility issues) before
releasing the `1.0` version of the `lv2` crate.

Host-only features will be gated behind a general `host`
[Cargo feature](https://doc.rust-lang.org/cargo/reference/features.html) and modules, as to not pollute the scope for
LV2 plugins.

Note that, when complete, this library will only provide APIs to allow hosts to instantiate and communicate with
LV2 plugins. It will not implement common LV2 host features such as plugin discovery, manifest parsing and such, like
the [Lilv](https://drobilla.net/software/lilv.html) C library does.

However, it is likely that such library will be implemented at some point, while still integrating with the `lv2` crate,
for a more complete LV2 Host development experience. It will also likely be developed under the same
[RustAudio](https://github.com/RustAudio) organization, possibly by the same authors.

## `#![no_std]` support

Technically, nothing in the LV2 APIs or specifications require any kind of Operating System support. Therefore, all LV2
APIs, including this crate, could be `#![no_std]`-compatible.

However, while running LV2 hosts and plugins is possible, it mostly seems like a curiosity at this point. Unlike DAW
support (which is already an established and common workflow), we consider `!#[no_std]` to be a nice-to-have, and we
have no intent to focus on it. However, it may come in future versions, and Pull Requests implementing it are always
welcome!
