# Kakpipe

![kakpipe](kakpipe.png?raw=true "colors in kakoune fifo buffer and info box")

`kakpipe` is a binary executable meant to be use with the included [kakoune](https://kakoune.org/) module
`kakpipe.kak`, to display text with ansi color codes inside fifo buffers or info boxes.

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

Defining a new command just for interfacing an external command to kakoune as described in
[interfacing](https://github.com/mawww/kakoune/blob/master/doc/interfacing.asciidocSometimes) feels cumbersome for
simple workflows, and as fifo doesn't support ansi-code yet you generally end up traveling back and forth between
kakoune and a shell just to launch a command that needs no interaction.

`kakpipe.kak` just define 2 kakoune commands built on top of `kakpipe`, you can use to automate those simples
workflows without leaving the comfort of your editor and without sacrificing readability:
- `kakpipe` which immediately switch to the buffer and let you see the result of the execution in real time with colors
   rendering and
- `kakpipe-bg` which just do everything in the background.

This utility would be voided if kakoune implements an `-ansi` argument on `edit -fifo` or `info` commands.

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

## Examples

### Buffers

Launch cargo build in a new fifo buffer

```
:kakpipe -S -- cargo build --color=always
```

Launch cargo build in a new fifo buffer in the background

```
:kakpipe-bg -- cargo build --color=always
```

Show a file with syntax coloring managed by [bat](https://github.com/sharkdp/bat)

```
:kakpipe -n main.rs -- bat -p --color=always src/main.rs
```

Show a rustdoc page in a buffer using [rusty-man](https://git.sr.ht/~ireas/rusty-man)

```
:kakpipe -- rusty-man --viewer rich std::string::String
```

### Info boxes

Show a calendar in an info box

```sh
:info -markup %sh{ TERM=xterm-256color cal --color=always | kakpipe faces }
```

Show diff of current file in info box

```sh
:info -markup %sh{ git diff --color=always $kak_buffile | kakpipe faces }
```

## Building new Commands

Mimicking shell commands inside kakoune are generally one-liners

```
define-command -override -params 1.. -docstring 'launch cargo with the given parameters inside kakoune' cargo %{
	kakpipe -S -- cargo %arg{@} --color=always
}
```

```
define-command -override -params 1 -docstring 'show a rustdoc page' rman %{
	kakpipe -n %arg{1} -- rusty-man --viewer rich %arg{@}
}
```

As well as for aliasing commands (shell like aliases)

```
define-command -params 0.. -docstring 'cargo check' cc %{
	cargo check %arg{@}
}
```

```
define-command -params 0.. -docstring 'cargo build' cb %{
	cargo build %arg{@}
}
```

```
define-command -docstring 'cargo install current directory crate to ~/.local/bin' ci %{
	cargo install --path . --root %sh{ echo -n ~/.local }
}
```


## References

[kak-ansi](https://github.com/eraserhd/kak-ansi) is a tiny (23K) executable (written in C with no dependencies)
also targeted at highlighting ansi-codes in buffers, but works by sending selections back and forth to kakoune
and use temporary files, where kakpipe use unix socket and in memory ring buffer. kak-ansi replaces ansi-codes from
a buffer, whereas kakpipe sends text without ansi-codes and provides range-specs on a separate unix socket to be
consumed inside kakoune hooks.
