# Contributing to Rust-LV2

*Inspired by [Rust's contribution guide](https://github.com/rust-lang/rust/blob/d9051341a1c142542a3f7dab509266606c775382/CONTRIBUTING.md).*

Thank you for your interest in contributing to Rust-LV2! There are many ways to contribute, and we appreciate all of them.

## General or Usage questions, Feature Ideas or Discussions

Our main discussion panel is the [Rust Audio Discourse](https://rust-audio.discourse.group/). Most of our design decisions are taken there and it's the perfect place to post any question or idea. Rust-LV2 isn't the only project there, so you may post anything that is somehow related to Rust and Audio Processing.

## Bug Reports

While bugs are unfortunate, they're a reality in software. We can't fix what we don't know about, so please report liberally. If you're not sure if something is a bug or not, feel free to file a bug anyway.

If you have the chance, before reporting a bug, please [search existing issues](https://github.com/RustAudio/rust-lv2/issues), as it's possible that someone else has already reported your error. This doesn't always work, and sometimes it's hard to know what to search for, so consider this extra credit. We won't mind if you accidentally file a duplicate report.

Similarly, to help others who encountered the bug find your issue, consider filing an issue with a descriptive title, which contains information that might be unique to it. This can be the used crates, the conditions that trigger the bug, or part of the error message if there is any.

Opening an issue is as easy as following [this link](https://github.com/RustAudio/rust-lv2/issues/new) and filling out the fields. Here's a template that you can use to file a bug, though it's not necessary to use it exactly:

``` MD
<short summary of the bug>

I tried this code:

<code sample that causes the bug>

I expected to see this happen: <explanation>

Instead, this happened: <explanation>

## Meta

### Dependencies:

    lv2-core = "1.0.0"
    ...

### Backtrace:
```

All three components are important: what you did, what you expected, what happened instead. Please include the dependencies to Rust-LV2 in your `Cargo.toml` so we can track down where to find the bug.

Sometimes, a backtrace is helpful, and so including that is nice. To get a backtrace, set the `RUST_BACKTRACE` environment variable to a value other than `0`. The easiest way to do this is to invoke `cargo` like this:

    $ RUST_BACKTRACE=1 cargo build

## Pull Requests

Pull requests are the primary mechanism we use to change Rust-LV2. GitHub itself has some [great documentation](https://help.github.com/articles/about-pull-requests/) on using the Pull Request feature. We use the "fork and pull" model [described here](https://help.github.com/articles/about-collaborative-development-models/), where contributors push changes to their personal fork and create pull requests to bring those changes into the source repository.

Please make pull requests against the `develop` branch.

Only pull requests that are up to date with the `develop` branch can be merged. Therefore, you should always use rebase to bring changes from the `develop` branch to your feature branch. Your changes also have to pass a check by Travis CI. This includes formatting checks using [rustfmt](https://github.com/rust-lang/rustfmt), coding style checks using [clippy](https://github.com/rust-lang/rust-clippy) and automated unit and integration tests. Your changes are checked on MacOS and Linux, using the stable, beta, and nightly versions of Rust. Although only the stable and beta versions are required to pass, you are also encouraged to make your changes work with nightly Rust. For more details, checkout our [Travis CI setup.](.travis.yml)

GitHub allows closing issues using keywords. This feature should be used to keep the issue tracker tidy. However, it is generally preferred to put the "closes #123" text in the PR description rather than the issue commit; particularly during rebasing, citing the issue number in the commit can "spam" the issue in question.

Another useful feature are [draft Pull Requests](https://github.blog/2019-02-14-introducing-draft-pull-requests/). If you want to let us know that you are working on something, but don't want us to pull it yet, you should create a draft pull request.

## Writing Documentation

Documentation improvements are very welcome. The API documentation is generated from the source code itself. Documentation pull requests function in the same way as other pull requests.

To find documentation-related issues, sort by the [documentation](https://github.com/RustAudio/rust-lv2/labels/documentation) label.