<!--
SPDX-FileCopyrightText: 2025 blinry <mail@blinry.org>

SPDX-License-Identifier: CC-BY-SA-4.0
-->

# ycwd – helps replace [xcwd](https://github.com/rhaberkorn/xcwd) on Wayland compositors

Given the process ID of a terminal emulator, returns its current working directory. This allows you to quickly open a new terminal in the same directory as the currently focused one.

Example usage:

```sh
$ ycwd 323994
/home/blinry/wip/ycwd
```

Specifically, ycwd returns the deepest child process of the given process that is still attached to a tty. This means that it will work for sub-shells in many scenarios.

## Installation

```
cargo install ycwd
```

## Setup

Look up how to start your terminal emulator in a specific directory. We're assuming that `$term` is your terminal emulator (examples: `kitty` or `foot`), and that it has a `--working-directory` option.

You will need to find a way to find the correct process ID, how to do that depends on your compositor:

### niri

Add a key binding like this (requires [jq](https://github.com/jqlang/jq)):

```
Mod+Return { spawn "bash" "-c" "$term --working-directory=\"$(ycwd $(niri msg --json focused-window | jq .pid))\""; }
```
### sway

Add a key binding like this (requires [jq](https://github.com/jqlang/jq)):

```
bindsym $mod+Return exec $term --working-directory="$(ycwd $(swaymsg -t get_tree | jq '.. | select(.type?) | select(.focused==true) | .pid'))"
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
