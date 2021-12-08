declare-option -hidden range-specs kakpipe_color_ranges

provide-module kakpipe %{

define-command -docstring "Forwards outputs of the command given as parameter to a new fifo buffer and highlights text based on ansi color codes" kakpipe -params 1.. %{
	evaluate-commands %sh{ exec kakpipe fifo -s $kak_session "$@" }
}

define-command -docstring "Forwards outputs of the command given as parameter to a new fifo buffer in the background and highlights text based on ansi color codes" kakpipe-bg -params 1.. %{
	evaluate-commands -draft %sh{ exec kakpipe fifo -s $kak_session "$@" }
}

}
