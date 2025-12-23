# Footnote

Footnote is a local-first personal wiki of markdown documents with support for
sync across your own devices and sharing with known, trusted sources.

## Core Concepts

### Personal Wiki

Your notes are a markdown wiki. Notes live in a vault and only ever exist on
your devices. They are only ever transmitted encrypted over the network, and
only transmitted to known devices and known peers.

### Trusted Sources

Your notes live in a vault with a special directory called `footnotes`. These
are documents you can link to that are owned and maintained by trusted sources.
Trust is established with a one-time, out-of-band setup process.

### Share vs Sync

"Share" is defined as transmitting documents to your trusted peers.

"Sync" is defined as coordinating an eventually consistent view of all your
notes across all your devices.

### Infrastructure

#### Public

The internet infrastucture required to facilitate data exchanges are Iroh
signaling servers. An end user just needs a device that is on somewhere.

#### Primary Device

A user designates a single device as primary. This is the device where the vault
is created. Additional devices are added via the primary. On the primary device,
the user creates a new device. This generates a one-time key. On a secondary
device, the user joins a repo, providing the one-time code.

It's not required that the primary device is always available, but a generally available
device would work well. A desktop, a laptop, even an old phone that stays plugged in could
work for this contact point. Generally it's a low amount of syncing for a small amount of
data. Sharing happens between users' primary devices.

#### Secondary Devices

Each user can have multiple devices (laptop, phone, tablet, desktop). Every device:

- Has its own Iroh endpoint ID for network connectivity
- Is cryptographically signed by the user's master identity key
- Has a human-friendly name (e.g., "laptop", "phone")
- Can be verified as belonging to that user

## Identity and Users

### Three-Layer Identity System

A contact in your contact list has three identifiers:

1. **Master Key** - Ed25519 public key, the primary identifier
2. **Nickname** - The name provided in their contact record.
3. **Petname** - The name you use for them locally. "mom", "dad", etc.

When you trust someone, you assign them a petname. That becomes their directory
name in `footnotes/`. Multiple people can refer to the same person by different
petnames, but all map to the same master key.

## Filesystem Layout

```
vault_root/
├── .footnotes/
│   ├── device_key               # Iroh SecretKey for this device
│   ├── id_key                   # Ed25519 master key (primary only)
│   ├── user.json                # Usrname and verified devices
│   └── contacts/
│       ├── alice.json           # Alice's contact record
│       └── bob.json             # Bob's contact record
├── home.md
├── inbox.md
├── outbox.md
└── interesting_ideas.md
└── footnotes/                   # Trusted users' shared notes
    ├── alice/
    │   ├── research.md
    │   └── thoughts.md
    └── bob/
        └── movie_list.md
```

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

I found [[footnotes/alice/research.md|Alice's research]] really insightful.

This connects to my earlier thoughts in [[interesting_ideas]].
```

### Key Fields

- **uuid**: Canonical identifier for the document (stable across renames/moves)
- **share_with**: Array of petnames for users who should receive this document

### Linking

- You can link to your own notes: `[[interesting_ideas]]`
- You can link to trusted sources: `[[footnotes/alice/research.md]]`

## Trust Relationships

Trust is managed through contact records in JSON format:

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

A user creates their own record by initializing a vault on disk, which writes
out the initial metadata for the primary device. Secondary devices join the
first device, building up the full contact record for a user. A user can then
share their contact record with a peer. A different user can then import that
document.

The contact record has a public key and checksum of the data tied to the public
key. All iroh connections are verified connections, tied to particular devices.
The user contact record ties together devices with a common public key.

## Future Work

### Major Missing Pieces

- Legitimate document merging (diff-patch-merge, CRDT)
- Contact distribution upon update
- Linking: Links are probably easiest if they link to a doc-uuid, but being able
  to use a local file system path is better, and being able to use the same path
  everywhere is even better. It'd be nice if Alice could reference
  footnotes/bob/events/party_at_my_place.md and Charlie, a friend of both, could
  navigate those documents easily on their local filesystem.

### General Improvements

- Better markdown support
- FZF/RG style document search

### Possible Improvements

- Shared primary, the ability for a family PC to be used for the whole family's notes
- Integration with a more sophisticated sharing, sync or permission system

## Design Philosophy

Overall, Footnote prioritizes simplicity and viability. Users should be able to
understand what they own, what they are sharing, who they are sharing with and
where their data lives.

## References

- https://www.inkandswitch.com/keyhive/notebook/
- https://files.spritely.institute/papers/petnames.html
