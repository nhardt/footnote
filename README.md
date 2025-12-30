# Footnote

Footnote is a local-first personal wiki of markdown documents with support for
sync across your own devices and sharing with known, trusted peers.

## Core Concepts

### Personal Wiki

Your notes are a markdown wiki. Notes live in a vault and only ever exist on
your devices. They are always encrypted in transit over the network, and
only transmitted to known devices and known peers.

### Trusted Sources

Your notes live in a vault with a special directory called `footnotes`. These
are documents you can link to that are owned and maintained by trusted sources.
Trust is established with a one-time, out-of-band setup process.

### Share vs Replica|Mirror|Outpost?

"Share" is defined as transmitting documents to your trusted peers. The view
of your documents is unique to each person you share with.

"Mirror" is defined as coordinating an eventually consistent view of all your
notes across all your devices, including things shared with you.

### Infrastructure

#### Public

The internet infrastucture required to facilitate data exchanges are Iroh
signaling servers. An end user just needs a device that is on somewhere.

#### Primary Device

A user designates a single device as primary. This is the device where the vault
is created. Additional devices are added via the primary. On the primary device,
the user creates a new device. This generates a one-time key. On a secondary
device, the user joins a vault, providing the one-time code.

It's not required that the primary device is always available, but a generally
available device would work well. A desktop, a laptop, even an old phone that
stays plugged in could work for this contact point if you are keeping together a
vault of text files.  Sharing happens between users' primary devices.

#### Secondary Devices

Each user can have multiple devices (laptop, phone, tablet, desktop). Every device:

- Has its own Iroh endpoint ID for network connectivity
- Is cryptographically signed by the user's master identity key
- Has a human-friendly name (e.g., "laptop", "phone")
- Can be verified as belonging to that user

## Identity and Users

### Three-Layer Identity System

A contact in your contact list has three identifiers:

- Master Key - Ed25519 public key, the primary identifier for a user
- Username: The name they chose for themselves. It is part of the crypto signature
- Nickname - The name you will attach to a document to share with them.

When you import a contact, you assign them a nickname. That becomes their
directory name in `footnotes/`. If you import Mom's contact record and use "mom"
for her nickname, the directory her shared files will be in is footnotes/mom/.

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

I found Alice's research[^1] really insightful.

This connects to my earlier thoughts in [[interesting_ideas]].

[^1] footnote.wiki://450332400-e29b-41d4-a716-446655440000

```

### Key Fields

- uuid: Canonical identifier for the document (stable across renames/moves)
- share_with: Array of petnames for users who should receive this document

### Linking

Linking builds on markdown's footnote style links.

## Trust Relationships

Trust is managed through contact records in JSON format:

```json
{
  "nickname": "@alice-jones",
  "username": "alice",
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

## Security

Layers of security:
- Device only listens when you want
- Connections come from endpoints verified by crypto key
- First step in connection is to connection vs imported contact
- Importing contacts is manual

## Todo 

### Little

- update display on file changes
- progress on sync
- fzf full text search
- Top level button "always on" sync
- debt: rework components to take the data they need, vs taking a path and getting it
- debt: factor tailwind heavy primitives (button, modal)

### Medium

- Always on mode for primary
- Sync to primary on save
- contact refresh
- local file rename 
- local file deletes
- Contact distribution upon update
- sync log

### Big

- replicate file deletes (probably can get by with a path, deleted timestamp)
- "as if" view. browser your files as if you are a user you share with
- drop drop sharing: in contact_view, ability to include/exclude files
- share with groups
- automated testing across supported platforms
- scale testing (targeting 200 peers max)

## Under Consideration (how/if/when)

- view/edit modes
- Linking: Links are probably easiest if they link to a doc-uuid, but being able
  to use a local file system path is better, and being able to use the same path
  everywhere is even better. It'd be nice if Alice could reference
  footnotes/bob/events/party_at_my_place.md and Charlie, a friend of both, could
  navigate those documents easily on their local filesystem. (if all links are
  by uuid, this might work)
- Shared primary, the ability for a family PC to be used for the whole family's notes
- better distrbuted writes
  - automerge, CRDT
  - diff-patch-merge https://crates.io/crates/diff-match-patch-rs
- Integration with a more sophisticated sharing, sync or permission system
- 2 way sync (maybe. maybe push only sync is actually a feature)

## Testing

There is cli testing for the core functionality in tests/ that uses the command
line. By hand testing has been done during development on:

- Linux
- android
- macos
- iphone

## References

- https://www.inkandswitch.com/keyhive/notebook/
- https://files.spritely.institute/papers/petnames.html
