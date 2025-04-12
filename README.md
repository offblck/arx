<div align="center">
    <img src="https://github.com/offblck/arx/blob/master/assets/example.png" />
</div>
<p align="center">ARX – a cli archive for all your bookmarks that you will totally read</p>
<p align="center"><img alt="Static Badge" src="https://img.shields.io/badge/version-0.1.0-blue"></p>

# A simple workflow

Adds a new entry. You can attach a category, url, status, tags, etc.
```
arx add "The C programming language" --category book
```
Shows your bookmarks (by default paginated with [-p page])
```
arx ls
```

Opens entry url in browser 🌐
```
arx open 3
```

Marks entry as done and hides it (like you'll ever need this... :smirk:)
```
arx done 2
```
# Installation
> Are we package yet?

Hell naw... 💔💔 arx is currently only available through release artifacts or built from source 😎 \
but I know you are goated so put it in a PATH folder or symlink it and enjoy!

# List of Commands
```
Commands:
  add       add bookmark
  list      list bookmarks (alias: ls)
  remove    remove bookmark (alias: rm, del, delete)
  open      open bookmark url in browser
  edit      edit bookmark
  done      mark bookmark as done
  copy-url  copy bookmark url (alias: cp)
  config    configure arx
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
You can get more detailed help messages for each subcommand `arx help edit` (`arx edit --help`)

# Configuration
arx is configurable from the cli and it is preferred that you configure it this way, e.g.

```
arx config --save-location <path>
```

List of configs:
- `--save-location <path>` – set the directory in which your data should be saved (arx moves the file for you)
- `--table-style <style>` – set the style of the displayed table \
  - values include all the variants found in [comfy_table](https://docs.rs/comfy-table/latest/comfy_table/presets/index.html) and it is set to "utf8_full" by default
- `--page-by <number>` – sets the number of entries to show at a time. default: 10

# Sync
The save location is configurable, but by default user data gets saved in a `bookmarks.json` file using the `directories` rust crate, meaning:

- on Linux inside $XDG_DATA_HOME/arx or $HOME/.local/share/arx
- on macOS inside $HOME/Library/Application Support/dev.offblck.arx
- on Windows inside {FOLDERID_RoamingAppData}\arx\data

Feel free to sync this folder with GitHub, Syncthing or your preferred synchronization tool

# Planned

- [ ] Extended configuration
- [ ] Upload to package registry (?)
- [ ] Built-in sync (?)
