# How to Install

Before installing, please ensure:

* [Oh My Zsh](https://ohmyz.sh/) is installed.
* Your current shell is **Zsh**.

---

## Install the Zsh Plugin

In your terminal, run:

```bash
./install.sh
```

This script will install JVCS's Zsh support files locally and prepare the prompt segment definitions for use by your theme.

---

## Configure Your Zsh Theme

Open your current Zsh theme file (e.g., `~/.oh-my-zsh/themes/xxx.zsh-theme`) and paste the following content in an appropriate location:

```bash
# ----------------------------------------------------- #
# DISPLAY_LEVEL
# FULL   = 127.0.0.1:25331/account/sheet
# NORMAL =                 account/sheet
# SHORT  =                         sheet
JVCS_VIEW='NORMAL'

# Customizable prompt segment elements
JVCS_PREFIX='['
JVCS_SPLIT='/'
JVCS_SUFFIX=']'

# JVCS_PROMPT_SEGMENT default style:
# [your_account/your_sheet]

# Append JVCS prompt segment
PROMPT+='${JVCS_PROMPT_SEGMENT}'
# ----------------------------------------------------- #
```

After saving, reload your terminal, or run:

```bash
source ~/.zshrc
```

---

## Notes

* `JVCS_VIEW` controls the view level displayed in the prompt.
* `JVCS_PREFIX / SPLIT / SUFFIX` are used to customize the appearance.
* `JVCS_PROMPT_SEGMENT` is only responsible for displaying the status and does not execute any logic.

No further configuration is required.
