# GitNest Directory Layout & Storage Schemas

```
~/.gitnest/
├── config.toml         # Application configuration (version, log level, ssh defaults)
├── accounts.json       # Array of registered GitHub identities (using key_id)
├── projects.json       # Mappings of absolute project root paths to account IDs
├── ssh/                # Generated Ed25519 SSH keys (0700 dir, 0600 keys)
│   ├── id_ed25519_user1
│   └── id_ed25519_user1.pub
└── logs/               # Daily rolling log files
    └── gitnest.log.YYYY-MM-DD
```

## `accounts.json` Schema

```json
{
  "accounts": [
    {
      "id": "uuid-v4-string",
      "name": "Octocat Dev",
      "email": "octocat@github.com",
      "github_username": "octocat",
      "provider": "github",
      "key_id": "id_ed25519_octocat",
      "created_at": "2026-08-04T12:00:00Z"
    }
  ]
}
```

## `projects.json` Schema

```json
{
  "projects": [
    {
      "path": "/absolute/path/to/project",
      "name": "project-name",
      "account_id": "uuid-v4-string",
      "mapped_at": "2026-08-04T12:00:00Z"
    }
  ]
}
```
