# Home Helper
Current behavior: Listens to, and prints out, hyprland events. When you enter a submap, a helix-like panel will open showing the next valid keybinds (filtered to ones with a description). Connects directly to hyprland's IPC sockets.

Future behavior: Keeps track of current state of monitors, workspaces, windows, etc... with lazy loading to minimize number of calls to hyprctl sockets. Properly positions, sizes, and styles the submap panel. Has scripts for use with rofi (which may need to communicate with the main daemon script).

Dependencies:

    - kitty (for the panel)
    - hyprland
