# Kakpipe

`kakpipe` is a binary executable meant to be used with the included [Kakoune](https://kakoune.org/) module
`kakpipe.kak` for launching external tools inside colorful FIFO buffers, or displaying text with ANSI colors in
info boxes.

![kakpipe](kakpipe.png?raw=true "colors in kakoune fifo buffer and info box")

[TOC]

```
kakpipe 0.5.7

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

## Simplify interface with external tools

Defining a new command for interfacing external tool with Kakoune as described in
[interfacing](https://github.com/mawww/kakoune/blob/master/doc/interfacing.asciidoc#interactive-output) is cumbersome
for simple workflows, and as FIFO buffer doesn't support ANSI codes, either you have the extra work of defining a new
file type and highlighting rules on top of some boilerplate code, or you have to accept to see everything in monochrome.

As a result you generally end up using a shell, traveling back and forth to Kakoune just to launch a command
because it's simpler. You loose at the same time the comfort of staying inside the editor for something that
sometimes needs attention but few interactions.

`kakpipe` tackles these difficulties and allows you to launch any external tool in colorful read-only FIFO buffer by
just giving the command to launch along its arguments.

## Usage

`kakpipe.kak` defines 2 Kakoune commands (one liner) built on top of `kakpipe fifo`

- `:kakpipe` immediately switch to the buffer and let you see the result of the execution in real time,
- `:kakpipe-bg` do the same without switching to the FIFO buffer

On the status line, `[fifo]` serves as an indicator to see if the process is still running.

You can quickly or fuzzily jump between the buffers, and inside a FIFO buffer created by `kakpipe` 2 commands speed
up your workflows even more comparing to using a shell :

- Closing the buffer with `:bd` stops `kakpipe` and the process,
- `:!!` stop (if still running) and restart the same command that created the current FIFO buffer.

You can now focus on :

- Adding new commands and aliases on top of `:kakpipe` to launch external tools inside Kakoune even faster,
- and/or adding behavior on the FIFO buffer, by defining a new type and some key mappings.

You can read the section about how to integrate `kakpipe` to your module below and look at the forked
[kakoune-cargo](https://gitlab.com/eburghar/kakoune-cargo) module to see how easy it is to simplify existing ones.

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

`kakpipe` command arguments are forwarded to `kakpipe fifo` executable, so you should use `--` to separate
arguments of the command from the executable ones in your scripts or at the command prompt.

Here are all the accepted arguments by `kakpipe fifo`

```
kakpipe 0.5.7

Usage: kakpipe fifo <cmd> [<args...>] [-c] [-w] [-S] [-d] -s <session> [-N <prefix>] [-n <name>] [-k] [-V <vars...>] [-D <opts...>]

Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes, then detach itself,
forward command output to the fifo, and serve range-specs definitions through a unix socket that can be consumed
to stdout with the `range-specs` subcommand.

Positional Arguments:
  cmd               command to spawn
  args

Options:
  -c, --close       close current buffer before starting kakpipe (used
                    internally by :!!)
  -w, --rw          turns the buffer editable. by default they are readonly
  -S, --scroll      scroll down fifo buffer as new content arrives
  -d, --debug       stderr goes to *debug* buffer instead of fifo
  -s, --session     kakoune session
  -N, --prefix      fifo buffer name prefix (default is the command name)
  -n, --name        fifo buffer name (default is prefix + temporary id)
  -k, --clear-env   clear environment
  -V, --vars        environment variables to set (NAME=VALUE) or to export
                    (NAME)
  -D, --opts        options to set in the buffer scope (NAME=VALUE)
  --help            display usage information
```

Launch `cargo build` in a new FIFO buffer

```
:kakpipe -S -- cargo build --color=always
```

Launch `cargo build` in a new FIFO buffer in the *background*

```
:kakpipe-bg -- cargo build --color=always
```

Show a file with syntax coloring managed by [bat](https://github.com/sharkdp/bat) in a FIFO buffer named `*main.rs*`

```
:kakpipe -n main.rs -- bat -p --color=always src/main.rs
```

Show a `rustdoc` page in a buffer using [rusty-man](https://git.sr.ht/~ireas/rusty-man)

```
:kakpipe -- rusty-man --viewer rich std::string::String
```

Launch a one-liner script

```
kakpipe -S -N alive -- sh -c 'while true; do echo -e "\e[32malive !"; sleep 1; done'
```

Launch a long-running process in a new buffer with the variable `FORCE_COLOR` exported.

```
:kakpipe -S -V FORCE_COLOR=true -- npm run dev
```

Launch `lualatex` each time the current file is modified using [`entr`](http://eradman.com/entrproject/)

```
:kakpipe -S -N lualatex -- sh -c "echo '%val{buffile}' | entr -nr lualatex '%val{buffile}'"
```

Closing the buffer will stop the process. You can also use `-k` to clean up the environment in conjunction with
`-V PATH` to reexport explicitly a variable.

### Info boxes

For info boxes you use the `kakpipe faces` binary inside shell expansions.

```
kakpipe 0.5.7

Usage: kakpipe faces

Forward stdin to stdout with ansi color codes converted to kakoune face definitions

Options:
  --help            display usage information
```

Show a calendar in an info box

```
:info -markup %sh{ TERM=xterm-256color cal --color=always | kakpipe faces }
```

Show diff of current file in info box

```
:info -markup %sh{ git diff --color=always $kak_buffile | kakpipe faces }
```

## Building new Commands

Mimicking shell commands inside Kakoune are generally one-liners.

```
define-command -override -params 1.. -docstring 'launch cargo with the given parameters inside kakoune' cargo %{
	kakpipe -S -- cargo --color=always %arg{@}
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
define-command -docstring 'cargo install in ~/.local/bin' ci %{
    cargo install --path . --root %sh{ echo -n ~/.local }
}
```

```
define-command -docstring 'cargo install current directory crate to ~/.local/bin' ci %{
	cargo install --path . --root %sh{ echo -n ~/.local }
}
```

## Integrate `kakpipe` to your module

You can easily add custom behavior to the FIFO buffer created by `kakpipe` by using one or several `-D name=value`
command line arguments to set up options values in the FIFO buffer scope.

You can for instance make a module defining custom mappings for a given file type and use `-D filetype=myfiletype`
with `kakpipe` inside the plugin to automatically set up the file type of the created FIFO buffer.

The `-n` options allows to use the same buffer (name) at each command invocation. By default, `kakpipe` open a new
buffer which name is a '`-`' separated string made of the command name (or the prefix given with `-N`) and a
random ID. The random ID is also used as a prefix for all temporary files that are generated in `/tmp/kakpipe/`
(socket, FIFO and PID files).

```
define-command -override -params 1.. -docstring 'launch cargo with the given parameters inside kakoune' cargo %{
	kakpipe -S -n cargo -D filetype=cargo -- cargo --color=always %arg{@}
}
```

You can see [a
patch](https://gitlab.com/eburghar/kakoune-cargo/-/compare/b15c75180e8c851c8687c90550746dfedceebbed...master?from_project_id=27156852&view=parallel)
which shows how to use `kakpipe` as a replacement of highlighter and `mkfifo` boilerplate in the
[kakoune-cargo](https://gitlab.com/Screwtapello/kakoune-cargo) plugin.

## References

[kak-ansi](https://github.com/eraserhd/kak-ansi) is a tiny (23K) executable (written in C with no dependencies)
exclusively targeted at highlighting ANSI codes in selections. `kak-ansi` works by removing ANSI codes from selections
and adding range-specs to bring color and faces, but as a consequence can only work on read-write buffers. It writes
to temporary files and adds its own (tiny) layer of boilerplate code to be used in your commands and FIFO.

`kakpipe` manage process lifecycle and sends asynchronously its output to the FIFO buffer already stripped out
of ANSI codes while providing range-specs from a Unix socket to be consumed separately. It works by default on
read-only buffers because this is what command outputs are expected to be.
