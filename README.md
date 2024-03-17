# chromazone (cz) 🎨

Chromazone is a [banger tune](https://www.youtube.com/watch?v=s1BVwsyznKw) by
Mike Stern and a terminal colorizer.

Why not [colorizer](https://github.com/kulinsky/colorizer) or
[pipecolor](https://github.com/dalance/pipecolor)? Because it features

* 📦️ fewest dependencies (two: [regex](https://crates.io/crates/regex) and [owo-colors](https://crates.io/crates/owo-colors))
* 📈 fewest memory allocations
* ✨ most color and effect combinations
* 📝 arguably the simplest configuration format

## CHANGELOG

See the [CHANGELOG](CHANGELOG.md) for a release history.

## Usage

For one-off uses, pipe some output into to the `cz` binary and pass regex
patterns and corresponding color and effect descriptions with the `-m` or
`--match` argument like so

```sh
cat README.md | cz -m "\[[^\[]*\]" red,underline -m "^# .*$" yellow,bold
```

which should give the following output

<img src="https://raw.githubusercontent.com/matze/chromazone/master/assets/screenshot.png">

To re-use style definitions, create a
`$HOME/.config/chromazone/chromazone.styles` configuration file and specify
sections for each style containing one or more match patterns and style
descriptions like this

```
[diff]
"^@@.*@@$" yellow
"^-.*" red
"^+.*" green
```

The style can then specified with the `-s` or `--style` argument

```sh
diff Cargo.toml Cargo.lock | cz -s diff
```

Note that you can still extend given styles with additional `-m` arguments.


### Style descriptions

Style descriptions are comma-separated lists of foreground colors (`black`,
`blue`, `cyan`, `green`, `magenta`, `purple`, `red`, `white` and `yellow`),
background colors (`b:black`, `b:blue`, `b:cyan`, `b:green`, `b:magenta`,
`b:purple`, `b:red`, `b:white` and `b:yellow`) and effects (`bold`, `italic`,
`underline` and `strike`).


## Installation

For now you need a Rust toolchain and install it via `cargo`:

```sh
cargo install chromazone
```

## License

[MIT](./LICENSE)
