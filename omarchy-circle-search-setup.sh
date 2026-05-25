#!/bin/sh
bind="bindd = SUPER SHIFT, PRINT, Circle to Search, exec, omarchy-circle-search"
config="$HOME/.config/hypr/hyprland.conf"

if [ ! -f "$config" ]; then
    echo "No hyprland.conf found at $config"
    exit 1
fi

if grep -Fqs "$bind" "$config"; then
    echo "Keybind already set — nothing to do"
else
    echo "$bind" >> "$config"
    echo "Added keybind to $config"
    echo "Reload Hyprland: hyprctl reload"
fi
