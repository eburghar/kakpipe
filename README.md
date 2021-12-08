# Kakpipe

![kakpipe](kakpipe.png?raw=true "colors in kakoune fifo buffer and info box")

`kakpipe` is a binary executable meant to be used with the included [kakoune](https://kakoune.org/) module
`kakpipe.kak`, to display text with ansi color codes inside fifo buffers or info boxes.

```
kakpipe 0.4.1
Utility to display text with ansi color codes inside kakoune fifo buffers or info boxes

USAGE:
    kakpipe <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    faces          Forward stdin to stdout with ansi color codes converted to kakoune face definitions
    fifo           Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes,
                   then detach itself, forward command output to the fifo, and serve range-specs definitions through
                   a unix socket that can be consumed to stdout with the `range-specs` subcommand
    help           Prints this message or the help of the given subcommand(s)
    range-specs    Consume all available range-specs up to a given selection range from a given unix socket
```

## Simplify interfacing of external tools

Defining a new command for interfacing external tools to kakoune as described in
[interfacing](https://github.com/mawww/kakoune/blob/master/doc/interfacing.asciidoc) looks like cumbersome for
simple workflows, and as fifo doesn't support ansi-code yet, you generally end up using a shell, traveling back
and forth to kakoune just to launch a command that needs no or few interactions.

`kakpipe.kak` defines 2 kakoune commands built on top of `kakpipe fifo` you can use to automate those simples
workflows without leaving the comfort of your editor and without sacrificing readability by loosing colors and faces:

- `kakpipe` which immediately switch to the buffer and let you see the result of the execution in real time with colors
   rendering and
- `kakpipe-bg` which just do the same without switching to the fifo buffer

Closing the buffer with `:bd` stops kakpipe and the process. You can (re)launch a long running process in
a background buffer and consult traces or messages with `ga` or `:b ..` before going back to edition very quickly.

You can also add behavior on the fifo buffer by defining a type and some key mapppings (see the section how to
integrate kakpipe to your module below).

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

`kakpipe` command arguments are forwarded to `kakpipe fifo` executable so you should use `--` to separate
arguments of the command from the executable ones in your scripts or at the command prompt.

Here are all the accepted arguments by the `kakpipe fifo`

```
kakpipe 0.4.1

Usage: kakpipe fifo <cmd> [<args...>] [-w] [-S] [-d] -s <session> [-N <prefix>] [-n <name>] [-k] [-V <vars...>] [-D <opts...>]

Return kakoune commands for opening a fifo buffer and initializing highlighters for ansi-codes, then detach itself, forward command output to the fifo, and serve range-specs definitions through a unix socket that can be consumed to stdout with the `range-specs` subcommand.

Positional Arguments:
  cmd               command to spawn
  args

Options:
  -w, --rw          turns the buffer editable. by default they are readonly
  -S, --scroll      scroll down fifo buffer as new content arrives
  -d, --debug       stderr goes to *debug* buffer instead of fifo
  -s, --session     kakoune session
  -N, --prefix      fifo buffer name prefix (default is the command name)
  -n, --name        fifo buffer name (default is prefix + args + timestamp)
  -k, --clear-env   clear environment
  -V, --vars        environment variables to set (NAME=VALUE) or export (NAME)
  -D, --opts        options to set in the buffer scope (NAME=VALUE)
  --help            display usage information
```

Launch `cargo build` in a new fifo buffer

```
:kakpipe -S -- cargo build --color=always
```

Launch `cargo build` in a new fifo buffer in the *background*

```
:kakpipe-bg -- cargo build --color=always
```

Show a file with syntax coloring managed by [bat](https://github.com/sharkdp/bat) in a fifo buffer named `*main.rs*`

```
:kakpipe -n main.rs -- bat -p --color=always src/main.rs
```

Show a rustdoc page in a buffer using [rusty-man](https://git.sr.ht/~ireas/rusty-man)

```
:kakpipe -- rusty-man --viewer rich std::string::String
```

Launch a long running process in a new buffer with the variable `FORCE_COLOR` exported. Closing the buffer will stop
the process. You can also use `-k` to cleanup the environment in conjunction with `-V PATH` to reexport explicitely a
variable.

```
:kakpipe -S -V FORCE_COLOR=true -- npm run dev
```

### Info boxes

For info boxes you use the `kakpipe faces` binary inside shell expansions.

```
kakpipe 0.4.1

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

Mimicking shell commands inside kakoune are generally one-liners.

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
define-command -docstring 'cargo install current directory crate to ~/.local/bin' ci %{
	cargo install --path . --root %sh{ echo -n ~/.local }
}
```

## Integrate `kakpipe` to your module

You can easily add custom behavior to the fifo buffer created by `kakpipe` by using one or several `-D name=value`
command line arguments to setup options values in the fifo buffer scope.

You can for instance make a module defining custom mappings for a given filetype and use `-D filetype=myfiletype`
with `kakpipe` inside the plugin to automatically setup the file type of the created fifo buffer.

The `-n` options allows to use the same buffer (name) at each command invocation. By default kakpipe always open
a new buffer which name is a '`-`' separated string made of the command name, the 1st argument, and a timestamp.

```
define-command -override -params 1.. -docstring 'launch cargo with the given parameters inside kakoune' cargo %{
	kakpipe -S -n cargo -D filetype=cargo -- cargo --color=always %arg{@}
}
```

You can see [how to use
kakpipe](https://gitlab.com/eburghar/kakoune-cargo/-/compare/b15c75180e8c851c8687c90550746dfedceebbed...master?from_project_id=27156852&view=parallel)
as a replacement of highlighter and mkfifo boilerplate in the
[kakoune-cargo](https://gitlab.com/Screwtapello/kakoune-cargo) plugin.

## References

[kak-ansi](https://github.com/eraserhd/kak-ansi) is a tiny (23K) executable (written in C with no dependencies)
also targeted at highlighting ansi-codes in buffers, but works by sending selections back and forth to kakoune
and use temporary files, where kakpipe use unix socket and in memory ring buffer. kak-ansi replaces ansi-codes from
a buffer, whereas kakpipe sends text without ansi-codes and provides range-specs on a separate unix socket to be
consumed inside kakoune hooks. kakpipe also works on readonly buffer because it doesn't alter content.
