# apticrate

A tool to query what crates, versions and features are available to you by installing packages on a Debian-based system where dependencies are packaged as `librust-*-dev`. This is helpful when you have configured cargo to use only local packages (see blow).

## Usage

`apticrate` itself requires no dependencies beyond the Rust standard library but it does rely on `apt-cache` and `dpkg` being available in your `PATH`. It may be run as an ordinary user.

Run the program with a filter as a parameter, or with no parameter to list all Rust crates. For example:

```
$ apticrate struct
struct-patch 0.4.1              --         librust-struct-patch-dev
struct-patch-derive 0.4.1       --         librust-struct-patch-derive-dev
structmeta 0.2.0                --         librust-structmeta-dev
structmeta-derive 0.2.0         --         librust-structmeta-derive-dev
structopt 0.3.26                installed  librust-structopt-dev
  deps for feat "color"         --         librust-structopt+color-dev
  deps for feat "debug"         --         librust-structopt+debug-dev
  deps for feat "default"       installed  librust-structopt+default-dev
  deps for feat "doc"           --         librust-structopt+doc-dev
  deps for feat "no_cargo"      --         librust-structopt+no-cargo-dev
  deps for feat "suggestions"   --         librust-structopt+suggestions-dev
  deps for feat "wrap_help"     --         librust-structopt+wrap-help-dev
  deps for feat "yaml"          --         librust-structopt+yaml-dev
structopt-derive 0.4.18         installed  librust-structopt-derive-dev
synstructure 0.12.3             installed  librust-synstructure-dev
  deps for feat "proc-macro"    installed  librust-synstructure+proc-macro-dev
synstructure_test_traits 0.1.0  --         librust-synstructure-test-traits-dev
```

## Installation

First ensure you have a Rust toolchain installed, either by installing the package `rust-all` or by using `rustup`. The `fossil` package is also required to check out.

```
fossil clone https://src.1.21jiggawatts.net/apticrate
cd apticrate
cargo install --path .
```

## Using cargo offline with apt packages

To instruct cargo to use only dependencies you've locally installed with apt instead of those on crates.io, set up your `.cargo/config.toml` file like this. For example, the one in your home directory `/home/user/.cargo/config.toml`.

```toml
[net]
offline = true

[source]

[source.apt]
directory = "/usr/share/cargo/registry"

[source.crates-io]
replace-with = "apt"
```

## Why though?

See the blog post which prompted this: [Rust without crates.io](https://thomask.sdf.org/blog/2023/11/14/rust-without-crates-io.html)

## Licence

`apticrate` is released under the MIT licence. See `LICENCE` for details.

## Contact

Feel free to send any questions or feedback via email: [tk@1.21jiggawatts.net](mailto:tk@1.21jiggawatts.net)
