#   ████████                ████████
# ██▒▒▒▒▒▒▒▒██            ██▒▒▒▒▒▒▒▒██
# ██        ▒▒██        ██▒▒        ██    █████  ██    ██   ██████   █████
# ██          ▒▒████████▒▒          ██    ▒▒▒██  ██    ██   ██████  ██████
# ██            ▒▒▒▒▒▒▒▒            ██       ██  ██    ██  ███▒▒▒█  █▒▒▒▒█
# ██                                ██       ██  ██    ██  ███   ▒  ████ ▒
# ██                                ██       ██  ██    ██  ███      ▒████
# ██         ████      ████         ██       ██  ▒██  ██▒  ███       ▒▒▒██
# ██         ████      ████         ██    █  ██   ██  ██   ███   █  ██  ██
# ██         ████      ████         ██    █  ██   ▒████▒   ▒██████  ██████
# ██         ▒▒▒▒      ▒▒▒▒   █     ██    ▒████    ▒██▒     ██████  ▒████▒
# ██                         ██     ██     ▒▒▒▒     ▒▒      ▒▒▒▒▒▒   ▒▒▒▒
# ██                ██████████      ██
# ██                                ██    JustEnoughVCS CommandLine
#   ████████████████████████████████      Zsh Plugin
#   ▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒

autoload -Uz add-zsh-hook

##################
### APPEARANCE ###
##################

JVCS_VIEW='NORMAL'
JVCS_PREFIX='['
JVCS_SPLIT='/'
JVCS_SUFFIX=']'

###################
### THEME READS ###
###################

JVCS_PROMPT_SEGMENT=''

######################
### INTERNAL STATE ###
######################

JVCS_DISPLAY=''
JVCS_WS=''
JVCS_ACCOUNT=''
JVCS_SHEET=''
JVCS_UPSTREAM=''

###################
### STATE LAYER ###
###################

jvcs_read_state() {
  JVCS_WS="$(jv _workspace_dir)"
  JVCS_ACCOUNT="$(jv _account)"
  JVCS_SHEET="$(jv _sheet)"
  JVCS_UPSTREAM="$(jv _upstream)"
}

##################
### VIEW LAYER ###
##################

jvcs_compute_view() {
  # Must be in a Workspace, otherwise do not display
  if [[ -z "$JVCS_WS" ]]; then
    JVCS_DISPLAY=''
    return
  fi
  JVCS_DISPLAY='1'
}

####################
### RENDER LAYER ###
####################

jvcs_render_prompt() {
  # Only set prompt segment if display is enabled
  if [[ -n "$JVCS_DISPLAY" ]]; then
    case "$JVCS_VIEW" in
      FULL)
        JVCS_PROMPT_SEGMENT="%{$fg[white]%}${JVCS_PREFIX}${JVCS_UPSTREAM}${JVCS_SPLIT}${JVCS_ACCOUNT}${JVCS_SPLIT}${JVCS_SHEET}${JVCS_SUFFIX} %{$reset_color%}"
        ;;
      NORMAL)
        JVCS_PROMPT_SEGMENT="%{$fg[white]%}${JVCS_PREFIX}${JVCS_ACCOUNT}${JVCS_SPLIT}${JVCS_SHEET}${JVCS_SUFFIX} %{$reset_color%}"
        ;;
      SHORT)
        JVCS_PROMPT_SEGMENT="%{$fg[white]%}${JVCS_PREFIX}${JVCS_SHEET}${JVCS_SUFFIX} %{$reset_color%}"
        ;;
      *)
        JVCS_PROMPT_SEGMENT=''
        ;;
    esac
  else
    JVCS_PROMPT_SEGMENT=''
  fi
}

####################
### ORCHESTRATOR ###
####################

jvcs_update() {
  jvcs_read_state
  jvcs_compute_view
  jvcs_render_prompt
}

#############
### HOOKS ###
#############

JVCS_NEED_REFRESH=0

jvcs_preexec() {
  case "$1" in
    jv\ status | \
    jv\ use* | \
    jv\ as* | \
    jv\ exit | \
    jv\ sheet\ use* | \
    jv\ sheet\ sheet\ exit | \
    jv\ account\ as*)
      JVCS_NEED_REFRESH=1
      ;;
  esac
}

jvcs_precmd() {
  [[ "$JVCS_NEED_REFRESH" -eq 1 ]] || return
  JVCS_NEED_REFRESH=0
  jvcs_update
}

jvcs_chpwd() {
  jvcs_update
}

add-zsh-hook preexec jvcs_preexec
add-zsh-hook precmd jvcs_precmd
add-zsh-hook chpwd jvcs_chpwd

###############################
### FALLBACK INITIALIZATION ###
###############################

jvcs_precmd_init() {
  [[ -n "${JVCS_INITIALIZED:-}" ]] && return
  JVCS_INITIALIZED=1
  jvcs_update
}

add-zsh-hook precmd jvcs_precmd_init
