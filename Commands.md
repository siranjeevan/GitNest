# GitNest CLI Commands & Alias Reference

| Full Command | Alias | Description |
| :--- | :--- | :--- |
| `gitnest init` | `gitnest i` | Initialize `~/.gitnest/` directory layout |
| `gitnest login` | `gitnest l` | Login to GitHub via OAuth Device Flow & configure SSH key |
| `gitnest accounts` | `gitnest a` | List registered accounts |
| `gitnest account remove <ID>` | `gitnest ar <ID>` | Remove an account identity |
| `gitnest project add` | `gitnest pa` | Map project directory to an account identity |
| `gitnest project remove` | `gitnest pr` | Unmap project directory |
| `gitnest projects` | `gitnest ls` | List mapped projects |
| `gitnest scan [PATH]` | `gitnest sc` | Recursively scan directory for Git repos & batch map |
| `gitnest current` | `gitnest c` | Show account identity mapped to current directory |
| `gitnest clone <URL>` | `gitnest cl` | Clone repository and map identity |
| `gitnest push [ARGS]...` | `gitnest p` | Ephemeral `git push` using mapped SSH identity |
| `gitnest pull [ARGS]...` | `gitnest pl` | Ephemeral `git pull` using mapped SSH identity |
| `gitnest doctor` | `gitnest dr` | Run complete system health diagnostics |
| `gitnest version` | `gitnest v` | Display GitNest, Rust, Git, OS, and path info |
| `gitnest export` | `gitnest ex` | Export backup archive `gitnest-backup.zip` |
| `gitnest import <ZIP>` | `gitnest im` | Restore backup archive from ZIP |
| `gitnest status` | `gitnest s` | Display rich system status and repo state |
