# API Design guidelines for the LV2 Rust crate

* Memory Safety
* C Parity
* Real-Time Safety
* Performance
* Extensibility and Modularity
* Ergonomics and Usability
* Correctness
* Ecosystem integration
* `#![no_std]` compatibility

## Memory safety

### Goal: Exposing Safe and Sound LV2 APIs

### Anti-Goal: Exposing *only* Safe LV2 APIs

### Anti-Goal: Making Plugins robust against incorrect Host implementations

### Anti-Goal: Making Plugins robust against incorrect description (`.ttl`) files

## C Parity

### Goal: LV2 Users must be able to program the same behavior in both C and Rust

### Goal: Making every official LV2 API accessible through the `lv2` crate

### Anti-Goal: Making every other LV2 API accessible through the `lv2` crate

## Real-Time Safety

### Goal: Making all APIs needed for processing Real-Time Safe

### Nice-to-have: Making as many APIs as possible Real-Time Safe

### Non-Goal: Making Real-Time Safe API wrappers for non-Real-Time Safe LV2 APIs

### Anti-Goal: Enforcing Real-Time Safety in user code

## Performance

### Goal: Making APIs needed for processing as *blazingly fast* as possible

### Nice-to-have: Making APIs needed for processing as lightweight as possible

### Nice-to-have: Making all APIs as fast and lightweight as possible

## Extensibility and Modularity

### Goal: Make every LV2 spec into a separate crate

### Goal: Enable implementing new LV2 specifications on top of `lv2_core`

### Goal: Make every extensible LV2 spec extensible by other crates

### Goal: Use Rust `features` to disable optional external dependencies (if any)

### Future Goal: Split host-specific features using a `host` feature

## Ergonomics and Usability

### Nice-to-have: Follow the Rust API Guidelines and design "Rusty" APIs

### Non-goal: Design APIs to be as user-friendly as possible

## Correctness

### Nice-to-have: Design misuse-resistant APIs

### Anti-goal: Design all APIs to be impossible to misuse

## Ecosystem integration

### Goal: Integrate tightly with `core` and `std` standard libraries

### Nice-to-have: Provide integrations with popular and/or appropriate Rust libraries

## `#![no_std]` compatibility

### Future Goal: Make lv2_core `#![no_std]` compatible

### Nice-to-have: Make as many specs as `#![no_std]` compatible as possible

