# Rust Cellular Automata
[![Crates.io](https://img.shields.io/crates/v/rust_ca)](https://crates.io/crates/rust_ca)[![Rust](https://github.com/hugcis/rust_ca/actions/workflows/rust.yml/badge.svg)](https://github.com/hugcis/rust_ca/actions/workflows/rust.yml)

This is a command line tool to simulate and search the space of simple cellular
automata (with relatively small neighborhoods).

The tool outputs CA simulations in the form of GIF files.

## Install the tool 

Run the following command to build:
```
cargo install rust_ca
```

To use the library in a Rust project add the following to your `Cargo.toml` dependencies:

```toml
[dependencies]
rust_ca = "0.2.1"
```

`

## Example

The following command will run a CA with a rule numbered `16855021099980290151`.
The initial configuration is specified  in `exploding.pat` (by default it is
random). The CA has 3 states and is 128x128 cells. We want the output GIF file to
represent 2400 time steps, but displaying only 1 in 10.
```
target/release/rust_ca -n 3 -f rules/3_states/16855021099980290151.map.comp \
-p patterns/exploding.pat -k 10 -s 128 --delay 0 -t 2400 
```
This results in the following `test.gif` file: 

![Example CA GIF](assets/test.gif)

### Generate random CA GIFs

**Warning:** This command create 200 GIFs for a total size of ~500MB.
``` sh
cargo build --release
mkdir rgen
for i in $(seq 0 200); do
target/release/rust_ca -n 4 -k 10 -s 128 --delay 0 -t 2400 --rotate 1 --symmetric > rgen/test_$i.gif
done;

```


## CLI
The CLI usage is: 
```
Rust CA 0.1.0
Hugo Cisneros <hmj.cisneros@gmail.com>
A CLI CA simulator. With no options, this runs a randomly sampled CA rule with 2 states for 50 steps
and outputs it as a gif file test.gif

USAGE:
    rust_ca [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
           --delay <delay>                    [default: 10]
        -f, --file <file>
            File to read a rule from. The file must contain a valid rule for the corresponding
            number of states

        --horizon <horizon>                [default: 1]
    -p, --pattern <pattern>
    -r, --rule-sampling <rule-sampling>
            [default: dirichlet] [possible values: uniform, dirichlet]

    -s, --size <size>                      The size of the 2D CA grid [default: 128]
    -k, --skip <skip>
            Steps to skip at every time step for the output [default: 1]

    -n, --states <states>                  Number of states of the CA [default: 2]
    -t, --steps <steps>                    Simulation time [default: 50]
```
