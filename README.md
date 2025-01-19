<!--
SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# ycwd – helps replace [xcwd](https://github.com/rhaberkorn/xcwd) on Wayland compositors

Given the process ID of a terminal emulator, returns its current working directory. This allows you to quickly open a new terminal in the same directory as the currently focused one.

Example usage:

```bash
$ ycwd 323994
/home/blinry/wip/ycwd
```

Specifically, ycwd returns the deepest child process of the given process that is still attached to a tty. This means that it will work for sub-shells in many scenarios.

## Installation

```
cargo install ycwd
```

## Setup

You will need to find a way to find the correct process ID, how to do that depends on your compositor:

### niri

Add a key binding like this:

```
Mod+Return { spawn "bash" "-c" "kitty --working-directory=$(ycwd $(niri msg --json focused-window | jq .pid))"; }
```

### sway

Courtesy of [this one-liner](https://www.reddit.com/r/swaywm/comments/jzolrq/how_do_i_get_the_pid_of_the_currently_focused/),
bound to a suitable key combination (and assuming your chosen terminal emulator accepts a `--working-directory` option, eg: `foot`):

```
bindsym $mod+Return exec $term --working-directory=$(ycwd $(swaymsg -t get_tree | jq '.. | select(.type?) | select(.focused==true) | .pid'))
```

### X.org

ycwd also works on X.org! Run it like this (requires [xdotool](https://github.com/jordansissel/xdotool)):

```sh
ycwd $(xdotool getwindowfocus getwindowpid)
```

#### XFCE

On XFCE specifically, configure a keybinding that runs

```sh
sh -c 'exec exo-open --launch TerminalEmulator --working-directory="$(ycwd $(xdotool getwindowfocus getwindowpid))"'
```

### …your Wayland compositor is missing?

I'd be happy about a pull request!
