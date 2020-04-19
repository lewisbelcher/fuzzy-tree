__fzsel() {
  local cmd="fztree"
  setopt localoptions pipefail no_aliases 2> /dev/null
  eval "$cmd" | while read item; do
    echo -n "${(q)item} "
  done
  local ret=$?
  echo
  return $ret
}

fztree-file-widget() {
  LBUFFER="${LBUFFER}$(__fzsel)"
  local ret=$?
  zle reset-prompt
  return $ret
}

zle     -N   fztree-file-widget
bindkey '^T' fztree-file-widget
