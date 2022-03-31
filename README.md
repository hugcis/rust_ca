# Rust Cellular Automata
[![Crates.io](https://img.shields.io/crates/v/rust_ca)](https://crates.io/crates/rust_ca)[![Rust](https://github.com/hugcis/rust_ca/actions/workflows/rust.yml/badge.svg)](https://github.com/hugcis/rust_ca/actions/workflows/rust.yml)

This is a command line tool to simulate and search the space of simple cellular
automata (with relatively small neighborhoods).

The tool outputs CA simulations in the form of GIF files.

## Install the tool 

Run the following command to install on your system:
```
cargo install rust_ca
```

To use the library in another Rust project add the following to your `Cargo.toml` dependencies:
```toml
[dependencies]
rust_ca = "0.2.1"
```

## Example

The following command will run a CA with a rule numbered `16855021099980290151`.
The initial configuration is specified  in `exploding.pat` (by default it is
random). The CA has 3 states and is 128x128 cells. We want the output GIF file to
represent 2400 time steps, but displaying only 1 in 10.
```
rust_ca -n 3 -f rules/3_states/16855021099980290151.map.comp \
-p patterns/exploding.pat -k 10 -s 128 --delay 0 -t 2400 
```
This results in the following `test.gif` file: 

![Example CA GIF](assets/test.gif)

### Generate random CA GIFs

This generates 200 distinct CA ran for 2400 steps (only showing one in 10) with
4 states.

**Warning:** This command create 200 GIFs for a total size of ~500MB.
``` sh
cargo build --release
mkdir rgen
for i in $(seq 0 200); do
rust_ca -n 4 -k 10 -s 128 --delay 0 -t 2400 --rotate 1 --symmetric \
    > rgen/test_$i.gif
done;

```


## CLI
The CLI usage is: 
```
Rust CA 0.2.2
Hugo Cisneros <hmj.cisneros@gmail.com>
A CLI CA simulator. With no options, this runs a randomly sampled CA rule with 2 states for 50 steps
and outputs it as a gif file `test.gif`

USAGE:
    rust_ca [OPTIONS]

OPTIONS:
        --delay <DELAY>
            [default: 10]

    -f, --file <FILE>
            File to read a rule from or write to. The file must contain a valid rule for the
            corresponding number of states

    -h, --help
            Print help information

        --horizon <HORIZON>
            [default: 1]

    -k, --skip <SKIP>
            Steps to skip at every time step for the output [default: 1]

    -n, --states <STATES>
            Number of states of the CA [default: 2]

    -o, --output <OUTPUT>
            A file to write the GIF to. Defaults to standard output

    -p, --pattern <PATTERN>


    -r, --rule <RULE>
            Specify one of the implemented CA rule [possible values: GOL]

        --rotate <ROTATE>
            [default: 0]

        --rule-sampling <RULE_SAMPLING>
            [default: dirichlet] [possible values: uniform, dirichlet]

    -s, --size <SIZE>
            The size of the 2D CA grid [default: 128]

        --symmetric
            Make the rule symmetric (this will also apply to rules passed as files)

    -t, --steps <STEPS>
            Simulation time [default: 50]

        --use-tiled
            Use a tiled CA (defaults to true when the size is a multiple of TILE_SIZE)

    -V, --version
            Print version information

    -w, --write-rule <WRITE_RULE>
            File to read a rule from or write to. The file must contain a valid rule for the
            corresponding number of states
```
