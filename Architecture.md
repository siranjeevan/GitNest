# GitNest Architecture Overview (Revision 2.0)

## Service Layer & Architecture

GitNest follows strict Clean Architecture and SOLID principles.

```
src/
├── main.rs                   # Entry point, CLI parsing, logger init, error printing
├── lib.rs                    # Library root exposing internal modules
├── cli/                      # CLI Interface layer (clap definitions, aliases & UI handlers)
├── services/                 # Business Logic Service Layer
│   ├── account_service.rs    # Account management
│   ├── project_service.rs    # Subdirectory root detection & recursive scanning
│   ├── git_service.rs        # Ephemeral env git command execution (push/pull/clone)
│   ├── ssh_service.rs        # Key generation, key_id resolution, ~/.ssh/ discovery
│   ├── doctor_service.rs     # System health diagnostics
│   └── backup_service.rs     # ZIP export and import manager
├── providers/                # Provider Abstractions
│   ├── trait.rs              # GitProvider trait definition
│   └── github.rs             # GitHub OAuth & SSH key upload
├── config/                   # Configuration Manager
├── domain/                   # Account, Project, and GitNestError entities
├── storage/                  # Atomic temp-file -> fsync -> rename writer + Keyring
├── logger/                   # Daily rolling log appender (~/.gitnest/logs/)
└── utils/                    # Subdirectory search & path normalization
```

## Security & Ephemeral Execution Model

1. **Zero Permanent Configuration**: Repositories are never modified. Git commands are run with temporary environment variables:
   ```bash
   GIT_SSH_COMMAND="ssh -i ~/.gitnest/ssh/key_id -o IdentitiesOnly=yes"
   GIT_AUTHOR_NAME="User Name"
   GIT_AUTHOR_EMAIL="user@example.com"
   ```
2. **Key ID Resolution**: Identifiers (`key_id`) are stored in `accounts.json` rather than absolute file paths. Keys are dynamically resolved relative to `~/.gitnest/ssh/`.
3. **Fsync Atomic JSON Storage**: JSON files are written to temporary files, `fsync`'d, and atomically renamed to prevent corruption.
