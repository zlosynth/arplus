# Development

## Environment setup

Follow [Rust's installation guide](https://www.rust-lang.org/tools/install).

Additionally, install the dependencies to build and test the project. Rust
version is pinned to prevent issues where after upgrade the binary gets bigger
or slower:

``` sh
rustup toolchain install 1.86.0-x86_64-unknown-linux-gnu
rustup target add thumbv7em-none-eabihf --toolchain 1.86.0
cargo +1.86.0 install cargo-binutils flip-link
```

## Formatting, linting, unit tests

Run formatting, linter and unit tests:

```sh
make
```

Slow unit tests are skipped by default. You can run them with:

```sh
SLOW=yes make
```

## Flash via ST-Link

This requires external probe, such as the ST LINK-V3 MINI. The benefit of this
approach is that it allows to stay connected to the module, read logs, run a
debugger, or execute tests on the module. Note that the module needs to be
powered while the probe is connected.

This project uses [probe-rs](https://github.com/probe-rs/probe-rs) to deal with
flashing. Start by installing its dependencies. For Fedora, it can be done by
running the following:

```sh
sudo dnf install libusbx-devel libftdi-devel libudev-devel
```

You may then install needed udev rules. See the [probe-rs getting
started](https://probe.rs/docs/getting-started/probe-setup/) to learn how.

Then install Rust dependencies of probe-rs:

```sh
cargo +1.86.0 install probe-rs --features cli
```

To flash the project, call this make target:

```sh
make flash
```

Logging level can be set using an environment variable:

```sh
DEFMT_LOG=info make flash
```

## Flash via DFU

Unlike ST-Link, DFU flashing does not require any external probe. Just connect
the module to your computer via a USB cable.

First, install [dfu-util](http://dfu-util.sourceforge.net/) and
[cargo-binutils](https://github.com/rust-embedded/cargo-binutils).
On Fedora, this can be done by calling:

```sh
sudo dnf install dfu-util
cargo +1.86.0 install cargo-binutils
rustup +1.86.0 component add llvm-tools-preview
```

Click the RESET button while holding the BOOT button of the Daisy Patch SM to
enter the bootloader. Then call this make target:

```sh
make flash-dfu
```

## Hardware diagnostics

Analyze input values read from the hardware in real time.

Before running an embedded test, first make sure to go through the guidance
given in [Flash via ST-Link](#flash-via-st-link).

```sh
make diagnostics
```

## Firmware size

Daisy Patch SM can fit up to 128 kB of firmware. It is important to make sure that
the firmware size stays slim and no bloat gets in.

Install needed tooling:

```sh
cargo +1.86.0 install cargo-bloat cargo-binutils
rustup +1.86.0 component add llvm-tools-preview
```

Run the following command often to make sure no unnecessary heavy dependencies
are brought in:

```sh
make bloat
```

## Gerbers, BOM and CPL

I extensivelly use <https://github.com/Bouni/kicad-jlcpcb-tools> to deal with
the matters listed in the title, and to prepare project for manufacture.

## User manual

The user manual is defined as a Scribus project under `manual/user`. To build
it, first install needed pre-requisites. On Fedora it can be done by running
the following:

```sh
sudo dnf install python inkscape scribus lilypond
```

To build the manual:

```sh
make manual/user
```

The built PDF is then available in `manual/user/manual.pdf`.

## Build manual

The build manual is defined in latex under `manual/build`. To build it, first
install needed pre-requisites. On Fedora it can be done by running the following:

```sh
sudo dnf install inkscape texlive-latex texlive-ec texlive-microtype texlive-pagecolor texlive-parskip texlive-titling texlive-hardwrap texlive-mdwtools texlive-tcolorbox
```

To build the manual:

```sh
make manual/build
```

The built PDF is then available in `manual/build/manual.pdf`.
