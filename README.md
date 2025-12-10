# Footnote

Footnote is a command-line, peer-to-peer sync and share engine for markdown
documents with YAML frontmatter. It enables you to maintain a personal
collection of notes that syncs across all your devices, while selectively
sharing specific notes with trusted contacts.

## Core Concepts

### Your Notebook

Your notes live in a `notes/` directory. All your devices mirror this complete
collection - every device has access to everything you've written.

### Trusted Sources

When you trust someone, you're allocating space in a `footnotes/{name}/`
directory on your filesystem. Their shared documents appear there, and you can
reference them in your own notes. You're treating them as a credible source
worth citing.

### The Trust Model

- **`footnote trust alice.json`** - Establishes the communication channel.
  You're importing Alice's contact information (her identity and devices) and
  creating space for her shared work.

- **`footnote share alice`** - Sends your shared documents to Alice. Only
  documents marked with `share_with: [alice]` will be transmitted.

This separation means:

1. Setting up trust is a one-time relationship establishment
2. Sharing is an ongoing, selective transmission of specific documents
3. You control what goes out; they control what goes out to you

## Identity and Users

### Three-Layer Identity System

Every user has three identifiers:

1. **Master Key** - Ed25519 cryptographic keypair, the canonical immutable identity
2. **Nickname** - What they call themselves publicly (e.g., "@alice-jones")
3. **Petname** - What YOU call them locally (e.g., "alice", "mom", "bob")

When you trust someone, you assign them a petname. That becomes their directory
name in `footnotes/`. Multiple people can refer to the same person by different
petnames, but all map to the same master key.

### Devices

Each user can have multiple devices (laptop, phone, tablet, desktop). Every device:

- Has its own Iroh endpoint ID for network connectivity
- Is cryptographically signed by the user's master identity key
- Has a human-friendly name (e.g., "laptop", "phone")
- Can be verified as belonging to that user

### Primary Device

Each user designates one device as primary. This device:

- Holds the master private key
- Can authorize new devices by signing them
- Handles user-to-user sharing (primary-to-primary sync)
- Is specified in the `primary_device_name` field in `contact.json`

All your devices mirror your complete note collection, but only the primary
device handles sharing with other users' primary devices.

## Filesystem Layout

```
vault_root/
├── .footnotes/
│   ├── this_device              # Iroh SecretKey for this device
│   ├── master_identity          # Ed25519 master key (primary only)
│   ├── contact.json             # Your contact record
│   └── contacts/
│       ├── alice.json           # Alice's contact record
│       └── bob.json             # Bob's contact record
├── home.md
└── interesting_ideas.md
└── footnotes/                   # Trusted users' shared notes
    ├── alice/
    │   ├── research.md
    │   └── thoughts.md
    └── bob/
        └── movie_list.md
```

### Key Principles

- **Your notes**: All markdown files outside of hidden directory and `footnotes/` syncs to all your devices
- **Trusted sources**: Each trusted user gets a `footnotes/{petname}/` directory
- **Ownership boundary**: Any device belonging to Alice can update `footnotes/alice/`
- **Verification**: Device signatures prove which user a device belongs to

## Document Format

### Note Files

```markdown
---
uuid: 550e8400-e29b-41d4-a716-446655440000
share_with:
  - alice
  - bob
---

# My Research Notes

I found [[id:{doc-uuid}|Alice's research]] really insightful.

This connects to my earlier thoughts in [[interesting_ideas]].
```

### Key Fields

- **uuid**: Canonical identifier for the document (stable across renames/moves)
- **share_with**: Array of petnames for users who should receive this document

### Linking

- Links use UUIDs internally, maintaining stability across renames
- You can link to your own notes: `[[interesting_ideas]]`
- You can link to trusted sources: `[[footnotes/alice/research.md]]`
- Different users can use different link text for the same document

## Commands

### Initial Setup

```bash
footnote init --username alice-jones --device-name laptop
```

Creates the vault structure on your primary device with:

- Master identity key generation
- Initial contact record
- Directory structure (`notes/`, `footnotes/`, `.footnotes/`)

### Adding Your Own Devices

**On your primary device:**

```bash
footnote mirror listen
# Outputs: iroh://abc123...?token=xyz789
```

**On your new device:**

```bash
footnote mirror from "iroh://abc123...?token=xyz789" --device-name phone
```

This:

1. Connects the new device to your primary
2. Primary signs the new device
3. Both update their contact records
4. New device receives your complete notes collection

### Trusting Another User

**Export your contact information:**

```bash
footnote user export > my-contact.json
```

**Import someone else's contact:**

```bash
footnote trust alice-contact.json --petname alice
```

This:

1. Verifies the cryptographic signatures in their contact record
2. Creates `.footnotes/contacts/alice.json`
3. Creates `footnotes/alice/` directory
4. Establishes the communication channel

### Sharing Documents

**Mark documents for sharing** (edit frontmatter):

```markdown
---
uuid: 550e8400-e29b-41d4-a716-446655440000
share_with:
  - alice
---
```

**Push shared documents:**

```bash
# Share with all trusted users
footnote share

# Share only with Alice
footnote share alice
```

This transmits documents marked for that user to their primary device, where they appear in `footnotes/{your-petname}/`.

## Sync Model

### Last-Write-Wins (LWW)

Footnote uses simple conflict resolution with vector timestamps:

- **Document checkpoint**: `{uuid}_{vector_timestamp}_{path}`
- Most recent write wins when devices sync
- No complex CRDTs in initial implementation

### Mirror Sync (Your Devices)

All your devices maintain a complete mirror of your `notes/` directory. When devices connect:

1. Exchange manifests of all document checkpoints
2. Request newer versions from each other
3. Update local collection to match latest writes

### Share Sync (Between Users)

When you run `footnote share alice`:

1. Your primary device collects all documents with `share_with: [alice]`
2. Connects to Alice's primary device
3. Transmits those documents
4. They appear in Alice's `footnotes/{your-petname}/` directory

Alice's devices then mirror that directory from her primary.

## Contact Records

Contact records are JSON files containing:

```json
{
  "username": "alice",
  "nickname": "@alice-jones",
  "master_public_key": "...",
  "primary_device_name": "laptop",
  "devices": [
    {
      "name": "laptop",
      "iroh_endpoint_id": "...",
      "timestamp": "..."
    },
    {
      "name": "phone",
      "iroh_endpoint_id": "...",
      "timestamp": "..."
    }
  ],
  "timestamp": "...",
  "signature": "..."
}
```

### Signature Verification

- The entire record (except signature) is signed by the master private key
- Recipients verify the signature using the master public key
- This proves the devices legitimately belong to the claimed user
- Enables trusted third-party contact sharing (you can export contacts you trust)

## Security Properties

### Verified

- Device signatures prove a device belongs to a user
- Identity keys are cryptographically unique
- No one can impersonate a user without their private key
- Signature chains are verifiable by anyone

### Trusted

- Out-of-band contact exchange (first meeting or export)
- Manual key rotation verification (if master key is lost)
- Directory-as-permission-boundary model

### Not Tracked

- Individual document ownership (the footnotes directory owns all its documents)
- Per-document permission levels (sharing is all-or-nothing per user)
- Edit history provenance (LWW with vector timestamps, no signatures per edit)

## Technical Stack

- **Rust** - Core implementation
- **iroh** (v0.95) - P2P networking with Iroh endpoint IDs
- **ed25519-dalek** - Cryptographic signatures for identity
- **clap** - CLI argument parsing
- **serde/serde_json/serde_yaml** - Serialization
- **tokio** - Async runtime

## Future Improvements

- [ ] CRDTs for collaborative document editing
- [ ] More sophisticated delegation models (non-primary device sharing)
- [ ] Shared document ownership between multiple users
- [ ] Mobile clients with native UI
- [ ] Conflict resolution UI for competing writes

## Design Philosophy

Footnote prioritizes:

1. **Human usability** - Petnames let you organize contacts naturally
2. **Selective sharing** - Share specific documents, not your entire collection
3. **Cryptographic verification** - Identity is provable, not trust-based
4. **Simplicity** - Directory structure encodes permissions
5. **Personal scale** - Optimized for small groups of trusted contacts

This is a tool for researchers, writers, and thinkers who want to maintain their
own knowledge base while selectively collaborating with trusted peers.

## References

https://www.inkandswitch.com/keyhive/notebook/
https://files.spritely.institute/papers/petnames.html
