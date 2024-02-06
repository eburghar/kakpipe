declare-option -hidden range-specs kakpipe_color_ranges
declare-option -hidden str kakpipe_args ""

provide-module kakpipe %{

define-command -docstring "
kakpipe [<options>] -- <command>: Forwards outputs of the command given as
parameter to a new fifo buffer and highlights text based on ansi color codes

Options:
	-c, --close      close current buffer before starting kakpipe
	-w, --rw         turns the buffer editable. by default they are readonly
	-S, --scroll     scroll down fifo buffer as new content arrives
	-d, --debug      stderr goes to *debug* buffer instead of fifo
	-s, --session    kakoune session
	-N, --prefix     fifo buffer name prefix (default is the command name)
	-n, --name       fifo buffer name (default is prefix + temporary id)
	-k, --clear-env  clear environment
	-V, --vars       environment variables to set (NAME=VALUE) or export (NAME)
	-D, --opts       options to set in the buffer scope (NAME=VALUE)
" kakpipe -params 1.. %{
	evaluate-commands %sh{ exec kakpipe fifo -s $kak_session "$@" }
}

define-command -docstring "
kakpipe-bg [<options>] -- <command>: Forwards outputs of the command given
as parameter to a new fifo buffer in the background and highlights text
based on ansi color codes

Options:
	-c, --close      close current buffer before starting kakpipe
	-w, --rw         turns the buffer editable. by default they are readonly
	-S, --scroll     scroll down fifo buffer as new content arrives
	-d, --debug      stderr goes to *debug* buffer instead of fifo
	-s, --session    kakoune session
	-N, --prefix     fifo buffer name prefix (default is the command name)
	-n, --name       fifo buffer name (default is prefix + temporary id)
	-k, --clear-env  clear environment
	-V, --vars       environment variables to set (NAME=VALUE) or export (NAME)
	-D, --opts       options to set in the buffer scope (NAME=VALUE)
" kakpipe-bg -params 1.. %{
	evaluate-commands -draft %sh{ exec kakpipe fifo -s $kak_session "$@" }
}

define-command -hidden -docstring "
Close buffer and restart kakpipe on a kakpipe created buffer
" kakpipe-restart %{
    evaluate-commands %sh{ test -n "$kak_opt_kakpipe_args" && eval exec kakpipe fifo -c "$kak_opt_kakpipe_args" || echo nop }
}

}
