# GitNest: One Workspace. Multiple Git Identities.

GitNest is a production-quality, cross-platform CLI tool for developers to manage multiple GitHub (and future provider) identities on the same machine without altering global Git configuration.

## Key Architectural Highlights

- **Zero Global Config Pollution**: Never modifies `~/.gitconfig` or `.git/config`. Passes SSH identity and author info via temporary environment variables (`GIT_SSH_COMMAND`, `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL`).
- **Clean Service Layer**: Decoupled domain business logic (`AccountService`, `ProjectService`, `GitService`, `SshService`, `DoctorService`, `BackupService`).
- **Provider Abstraction**: Extensible `GitProvider` trait (`GitHubProvider`, ready for GitLab, Bitbucket, Gitea).
- **Subdirectory Root Detection**: Automatically detects `.git` repository root from any deeply nested subdirectory.
- **Key ID SSH Resolution**: Accounts store key identifiers (`key_id`) allowing portable `~/.gitnest/` relocation.
- **Repository Scanner**: Multi-repository scanning and batch identity mapping (`gitnest scan`).
- **Health Diagnostics**: Complete system diagnostics (`gitnest doctor`).
- **Backup & Restore**: Zip archiving (`gitnest export` & `gitnest import`).
- **Short Aliases**: Full CLI support for `p`, `pl`, `c`, `s`, `ls`, `a`, `pr`, `dr`, `sc`, `i`, `l`, `v`, `ex`, `im`.

---

## Command Reference & Short Aliases

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

---

## Installation & Usage

```bash
cargo build --release
cargo install --path .

gitnest init
gitnest login
gitnest scan ~/development
gitnest status
```
