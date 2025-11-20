# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Fieldnote is a P2P sync and share engine for markdown notes with YAML frontmatter. It uses Iroh for networking and implements a cryptographic identity system based on the petnames concept for managing users and devices.

**Key Concept:** Users share notes through "embassies" - dedicated directories where another user's shared content is mirrored. Any device belonging to a user can update their embassy on your filesystem, with the directory itself serving as the permission boundary.

## Build and Test Commands

```bash
# Build the project
cargo build

# Build release
cargo build --release

# Run the CLI
cargo run -- <subcommand>

# Run tests (shell integration tests)
./tests/integration_test.sh
./tests/hq_create_test.sh
./tests/hq_outpost_test.sh
```

## Development Commands

```bash
# Test HQ creation
cargo run -- hq create --username testuser --device-name laptop

# Test user export/import flow
cargo run -- user export me > contact.yaml
cargo run -- user import contact.yaml --petname friend

# Test device authorization (requires two terminal sessions)
# Terminal 1 (primary device):
cargo run -- device create
# Terminal 2 (remote device):
cargo run -- device create remote "iroh://..." --device-name phone
```

## Code Architecture

### Module Structure

- **`src/cli/`** - Command-line interface definitions using clap
  - `commands.rs` - CLI command structure and routing to core functions

- **`src/core/`** - Core business logic
  - `crypto.rs` - Ed25519 identity keys, device signatures, contact record signing/verification
  - `device.rs` - Device creation and authorization (primary and remote joining)
  - `hq.rs` - Headquarters (primary device) initialization
  - `user.rs` - User management, contact export/import
  - `vault.rs` - Filesystem layout and path management
  - `mirror.rs` - Mirroring/sync operations between devices
  - `sync.rs` - Sync protocol implementation
  - `note.rs` - Note/document handling
  - `manifest.rs` - Document manifest for sync

### Filesystem Layout

The vault structure created by `fieldnote hq create`:

```
vault_root/
├── .fieldnotes/
│   ├── this_device           # Iroh SecretKey for this device
│   ├── master_identity       # Ed25519 master private key (HQ only)
│   └── contact.json          # Signed contact record with all devices
├── identity.md               # User's public identity and nickname
├── notes/                    # This user's notes
│   └── home.md
└── embassies/                # Other users' shared content
    ├── bob.json              # Bob's contact record (signed)
    └── bob/
        └── notes/            # Bob's shared notes (mirror)
```

**Important:** The vault is located by searching upward from the current directory for `.fieldnotes/`. Most commands expect to be run from within a vault.

### Identity and Cryptography

**Three-layer identity system:**
1. **Master Key** - Ed25519 keypair, canonical identity (in `.fieldnotes/master_identity`)
2. **Nickname** - User-chosen global name (in `identity.md` frontmatter)
3. **Petname** - Local name you assign to others (directory name in `embassies/`)

**Device Authorization:**
- Each device has an Iroh endpoint ID for networking
- Devices are signed by the user's master identity key
- Contact records (`contact.json`) contain all devices + signature
- Remote peers verify signatures to authenticate devices

**Contact Records:**
- JSON format containing username, nickname, master public key, devices array, timestamp, and signature
- Signed by the master identity key using `crypto::sign_contact_record()`
- Verified using `crypto::verify_contact_signature()`

### Sync Model

**Last-Write-Wins (LWW)** with vector timestamps:
- Document checkpoint: `{uuid}_{vector_timestamp}_{path}`
- UUIDs identify documents (stable across renames/moves)
- Sharing: Add user petnames to `share_with` array in frontmatter
- Embassy sync: When a device connects, it verifies device signature, then can read/write to its user's embassy directory

## Key Implementation Patterns

### Vault Path Resolution

Commands use `vault::get_vault_path()` to locate the vault by searching upward for `.fieldnotes/`. This is verified with `vault::verify_vault_layout()` before operations that need it.

### Device Creation Flow

**Primary device generating join URL:**
1. Starts listening with Iroh
2. Outputs connection URL: `iroh://endpoint-id?token=xyz`
3. Waits for remote device to connect

**Remote device joining:**
1. Takes connection URL from primary
2. Connects, exchanges identity info
3. Primary signs the new device
4. Both update their `contact.json` with new device + signature

### Contact Import/Export

Export produces a signed JSON structure with master public key and all device records. Import verifies signatures cryptographically and stores in `embassies/{petname}.json`. This enables trusted third-party contact sharing without manual device exchanges.

## Technical Dependencies

- **iroh** (v0.95) - P2P networking
- **ed25519-dalek** - Cryptographic signatures for identity
- **clap** - CLI argument parsing
- **serde/serde_json/serde_yaml** - Serialization for config and notes
- **tokio** - Async runtime

## Important Design Decisions

1. **Directory-as-permission-boundary:** Rather than per-document ownership tracking, the `embassies/{user}/notes/` directory itself defines what that user's devices can modify.

2. **LWW conflict resolution:** Simple last-write-wins with vector timestamps. No CRDTs in initial implementation.

3. **Vault-relative operations:** Most operations search for `.fieldnotes/` upward from pwd rather than requiring global config.

4. **Contact record signatures:** All device additions update and re-sign the entire contact record, ensuring tamper-evidence.

5. **Ed25519 for identity, Iroh keys for networking:** Separate concerns - master identity keys are long-lived and sign devices; Iroh endpoint keys handle network connectivity.

## Important User Instructions

1. **No Emojis** Do not include Emojis in code, stick to the ASCII character set.

2. **Self documenting code** Do not include comments in the code that only explain what the next line of code does