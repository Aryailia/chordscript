#!/usr/bin/env sh
outln() { printf %s\\n "$@"; }

#!/usr/bin/env sh

NAME="$( basename "$0"; printf a )"; NAME="${NAME%?a}"

show_help() {
  <<EOF cat - >&2
SYNOPSIS
  ${NAME}

DESCRIPTION
  

OPTIONS
EOF
}

ALT='Mod1+'
CTRL='Control+'
SHIFT='Shift+'
SUPER='Mod4+'

main() {
  case "${1}"
    in api)
      if [ -z "${2}" ]; then
        shift 2
        process_keyspace '' "$@"
      else
        domainify_title "${2}"
        printf '\nmode "%s" {\n' "${title}"
        shift 2

        process_keyspace '  ' "$@"
        printf '}\n'
      fi

    ;; bench)
      counter="0"
      while [ "${counter}" -lt 100 ]; do
        domainify_hotkey "Super + Ctrl + A" false 'A-' 'C-' 'S-' 'M-'
        outln "${hotkey}"

        counter="$(( counter + 1 ))"
      done
    ;; help) show_help; exit 0
    ;; *)
      cargo run ./config.txt keyspace-list "./${NAME}" api
  esac
}

# Passes data via setting "${title}" (and "${hotkey}")
domainify_title() {
  domainify_hotkey "${1}" false 'A-' 'C-' 'S-' 'M-' # sets ${hotkey}
  title="${hotkey}"
}

process_keyspace() {
  # Sets ${hotkey}
  _padding="${1}"
  shift 1
  while [ "$#" -gt 0 ]; do
    if [ "${2}" = "state" ]; then
      domainify_title "${3}" # Sets ${hotkey} and ${title}
      # Sets ${hotkey}
      domainify_hotkey "${1}" true "${ALT}" "${CTRL}" "${SHIFT}" "${SUPER}"
      printf '%sbindsym %s mode "%s"\n' "${_padding}" "${hotkey}" "${title}"
    else
      # Sets ${hotkey}
      domainify_hotkey "${1}" true "${ALT}" "${CTRL}" "${SHIFT}" "${SUPER}"
      printf '%sbindsym %s exec --no-startup-id "%s"\n' \
        "${_padding}" "${hotkey}" "${3}"
    fi
    shift 3
  done
}


# Passing data via variable to avoid subshell
# Speed over clairty as this gets called very often
domainify_hotkey() {
  # $1: chords to process
  # $2: 'true' to replace the key with the domain-specific version
  # $3: the domain-specific version of 'alt'
  # $4: the domain-specific version of 'ctrl'
  # $5: the domain-specific version of 'shift'
  # $6: the domain-specific version of 'super'
  _input="${1}"
  _replace_key="${2}"
  hotkey=""
  m=""
  while [ -n "${_input}" ]; do
    chord="${_input%%;*}"
    _input="${_input#"${chord}"}"
    _input="${_input#;}"

    # Trim it it without calling subshell (performance)
    while [ "${_input}" != "${_input# }" ]; do _input="${_input# }"; done
    while [ "${_input}" != "${_input% }" ]; do _input="${_input% }"; done

    k="${chord}"
    for _ in 1 2 3 4; do
      if   [ "${k}" != "${k#Alt + }" ];   then k="${k#Alt + }";   m="${3}${m}"
      elif [ "${k}" != "${k#Ctrl +}" ];   then k="${k#Ctrl + }";  m="${4}${m}"
      elif [ "${k}" != "${k#Shift + }" ]; then k="${k#Shift + }"; m="${5}${m}"
      elif [ "${k}" != "${k#Super + }" ]; then k="${k#Super + }"; m="${6}${m}"
      fi
    done

    if "${_replace_key}"; then
      case "${k}"
        in a|b|c|d|e|f|g|h|i|j|k|l|m|n|o|p|q|r|s|t|u|v|w|x|y|z) # do nothing
        ;; 0|1|2|3|4|5|6|7|8|9|10) # do nothing
        ;; Return)
        ;; Space) k="space"
        ;; *) die FATAL 1 "Unsupported key: ${k}"
      esac
    fi

    if [ -z "${hotkey}" ]; then
      hotkey="${m}${k}"
    else
      hotkey="${hotkey};${m}${k}"
    fi
  done
}

trim() {
  while [ "${1}" != "${1# }" ]; do set -- "${1# }"; done
  while [ "${1}" != "${1% }" ]; do set -- "${1% }"; done
  out "${1}"
}

# Helpers
out() { printf %s "$@"; }
outln() { printf %s\\n "$@"; }
errln() { printf %s\\n "$@" >&2; }
die() { printf %s "${1}: " >&2; shift 1; printf %s\\n "$@" >&2; exit "${1}"; }
eval_escape() { <&0 sed "s/'/'\\\\''/g;1s/^/'/;\$s/\$/'/"; }

main "$@"
