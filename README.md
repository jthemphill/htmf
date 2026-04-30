# Hey, that's my fish!

[![WWW build](https://github.com/jthemphill/htmf/actions/workflows/www-build.yml/badge.svg)](https://github.com/jthemphill/htmf/actions/workflows/www-build.yml)
[![PR Preview](https://github.com/jthemphill/htmf/actions/workflows/pr-preview.yml/badge.svg)](https://github.com/jthemphill/htmf/actions/workflows/pr-preview.yml)

## Simple webapp version of a deceptively simple children's game

You can play a productionized Webassembly version of the game at
https://jthemphill.github.io/htmf

## How to run in Webassembly mode

When running in Webassembly mode, the AI runs entirely locally on the client.
This is suitable for a "static" site like a Github page, where you're allowed to
ship HTML/CSS/JS to a client but aren't allowed to consume server resources.

You will need `bun` in order to install this game's packages and build this game's TypeScript.

To build automatically, download and run [just](https://github.com/casey/just):

```sh
just dev
```

To run all tests:

```sh
just test
```

## Bazel Rust builds

The Rust crates can also be built and tested with Bazel. Use
[Bazelisk](https://github.com/bazelbuild/bazelisk) or Bazel matching the
version in `.bazelversion`.

```sh
just build_rust
just test_rust
just build_wasm
```

`just sync_wasm_pkg` copies the Bazel-built WebAssembly package into `wasm/pkg`
so the existing Bun workspace import of `htmf-wasm` continues to work.

## Pull Request Previews

When you open a pull request, a preview of your changes will be automatically deployed to GitHub Pages. The preview URL will be posted as a comment on your PR, allowing reviewers to test your changes before merging.

Preview URLs follow the format: `https://jthemphill.github.io/htmf/pr-preview/pr-{number}/`

The preview will be automatically updated when you push new commits, and cleaned up when the PR is closed.
