# Hey, that's my fish!

[![WWW build](https://github.com/jthemphill/htmf/actions/workflows/www-build.yml/badge.svg)](https://github.com/jthemphill/htmf/actions/workflows/www-build.yml)

## Simple webapp version of a deceptively simple children's game

You can play a productionized Webassembly version of the game at
https://jthemphill.github.io/htmf

## How to run in Webassembly mode

When running in Webassembly mode, the AI runs entirely locally on the client.
This is suitable for a "static" site like a Github page, where you're allowed to
ship HTML/CSS/JS to a client but aren't allowed to consume server resources.

You will need `bun` in order to install this game's packages and build this game's TypeScript.

To start a Vite devserver:

```sh
bun dev
```

To run all tests:

```sh
bun test
```
