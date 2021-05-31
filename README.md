# Kakpipe

`kakpipe` is a binary executable and a module for kakoune to display text with ansi color codes inside fifo buffer
or info boxes. It works in 3 modes:

```
Usage: kakpipe <command> [<args>]

Utility to display text with ansi color codes inside kakoune fifo buffers or info boxes

Options:
  --help            display usage information

Commands:
  fifo              Return kakoune commands for opening a fifo buffer and
                    initializing highlighters for ansi-codes, then detach
                    itself, forward command output to the fifo, and serve
                    range-specs definitions through a unix socket that can be
                    consumed to stdout with the `range-specs` subcommand.
  range-specs       Consume all available range-specs up to a given selection
                    range from a given unix socket.
  faces             Forward stdin to stdout with ansi color codes converted to
                    kakoune face definitions
```

This utility would be voided if kakoune implements an `-ansi` argument on `edit -fifo` or `info` commands.

It uses a forked [yew-ansi](https://github.com/eburghar/yew-ansi.git) crate for parsing the ansi-codes to which I just
added support for `reversed` and `dimmed` ansi-codes that can be used in `kakoune` face definitions.

## Installation

### manual

Install `kakpipe` somewhere within you `$PATH`

```sh
cargo install --path . --root ~/.local
```

Copy `kakpipe.kak` in your autoload directory. Then enter in command prompt

```
:require-module kakpipe
```

### with plug.kak

with [plug.kak](https://github.com/andreyorst/plug.kak)

```
plug "eburghar/kakpipe" do %{
	cargo install --force --path . --root ~/.local
}
```

## Usage

Launch cargo in a new fifo buffer

```
:kakpipe -S -- cargo build --color=always
```

Launch cargo in a new fifo buffer in the background

```
:kakpipe-bg -- cargo build --color=always
```

Shows a file which syntax coloring is handled by bat

```
:kakpipe -n main.rs -- bat -p --color=always src/main.rs
```

Show a file which syntax coloring is handled by bat in an info box

```sh
:info -markup %sh{ bat -p --color=always src/lib.rs | kakpipe faces }
```

From there you can easily define new commands to eliminate travels between kakoune and terminal.

```
define-command -override -params 1.. -docstring 'launch cargo with the given parameters inside kakoune' cargo %{
	kakpipe -S -- cargo %arg{@} --color=always
}

define-command -params 0.. -docstring 'cargo check' cc %{
	evaluate-commands cargo check %arg{@}
}

define-command -params 0.. -docstring 'cargo build' cb %{
	evaluate-commands cargo build %arg{@}
}

define-command -override -params 1.. -file-completion -docstring 'launch bat in a fifo buffer' bat %{
	kakpipe -S -- bat -p --color=always %arg{@}
}
```

## References

[kak-ansi](https://github.com/eraserhd/kak-ansi) is an ultasmall (23K) executable (written in C with no dependencies)
also targeted at highlighting ansi-codes in buffers, but works by sending selections back and forth to kakoune
and use temporary files where kakpipe use unix socket and in memory ring buffer. kak-ansi replaces ansi-codes from
a buffer, whereas kakpipe sends text without ansi-codes and provides range-specs on a separate unix socket to be
consumed inside kakoune hooks.
