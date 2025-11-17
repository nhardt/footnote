# Fieldnote Users and Identity Design

## Overview

Fieldnote uses a cryptographic identity system where users are represented by
signing keys, and each user can have multiple devices. This design enables
secure peer-to-peer synchronization while maintaining human-readable names
locally through the petnames system.

## Identity Model

### The Three-Layer Petnames System

Every user in Fieldnote is represented through three distinct identifiers:

**1. The Key (Global, Immutable)**

- A cryptographic public key (Ed25519) that uniquely identifies the user
- This is the canonical, verifiable identity
- Never changes unless the user explicitly rotates their identity
- In Fieldnote, this is the user's master identity key

**2. The Nickname (Global, Human-Friendly)**

- What the user calls themselves publicly (e.g., "@alice-jones")
- User-chosen and potentially shared across their network
- Mutable - users can change how they present themselves
- Helps with discovery and reference

**3. The Petname (Local, Personal)**

- What YOU call the user in your system (e.g., "mom", "joe", "sister")
- Completely local to your device
- You choose this based on your relationship
- Can be different for each person who knows the same user

### Example

Three people can refer to the same individual:

- You call him: "joe" (petname)
- His mother calls him: "babyboy" (petname)
- He calls himself: "@joe-smith" (nickname)
- His cryptographic identity: `key-abc123...` (the key)

All three petnames map to the same underlying identity key, ensuring that when
you and his mother both link to his documents, you're linking to the same
person.

## Users and Devices

### One User, Multiple Devices

A user is a person with a master identity key. Each user can have multiple
physical devices (laptop, phone, tablet, desktop). Each device:

- Has its own Iroh endpoint ID for network connectivity
- Is cryptographically signed by the user's master identity key
- Can be verified by anyone as belonging to that user
- Has a human-friendly device name (e.g., "laptop", "phone")

### Device Authorization

When a user adds a new device, they create a signed device record that includes:

- The user's master public key (who owns this device)
- The device's Iroh endpoint ID (network address)
- The device name (for human reference)
- A timestamp
- A cryptographic signature proving the master key authorized this device

Remote peers can verify this signature to confirm a connecting device
legitimately belongs to the claimed user.

### The Primary Device

In Fieldnote's initial implementation, one device per user is designated as the
"primary device." This device:

- Holds the master private key
- Can authorize new devices by signing them
- Handles user-to-user synchronization
- Manages device additions for that user

The primary device designation is an optimization for stability (typically a
desktop or laptop that's frequently online) rather than a strict requirement.
Sync can occur between any devices, but routing through primary devices creates
more reliable connections.

The system is designed to potentially support more sophisticated delegation
models in the future, but the initial implementation uses the simpler
primary-device-only model.

## Mirror Outposts

When you add another user to your Fieldnote system, their shared documents
appear in a "mirror outpost" on your filesystem. This is a dedicated directory
for that user's shared content.

### Directory Structure

```
me/
  devices/
  notes/
bob/
  devices/
  notes/
charlie/
  devices/
  notes/
```

Each user directory represents a mirror outpost - a space on your filesystem
that reflects the documents that user has shared with you.

### Ownership and Authority

The key principle:

**Any device belonging to a user can update that user's mirror outpost on your system.**

When a device connects:

1. Your system verifies the device's signature chain
2. If verified, the device is authenticated as belonging to user X
3. That device can read and write any documents in the `X/notes/` directory
4. The directory itself is the permission boundary

This means:

- You don't track ownership per-document
- You don't need complex permission systems
- The filesystem layout encodes the security model
- Updates are accepted based on "which outpost does this device control?"

### Sharing Documents

When you want to share a document, you mark it in the frontmatter:

```markdown
---
uuid: 1234-5678
share_with:
  - bob
  - charlie
---
```

When your device syncs:

- Bob's devices receive the document in their `you/notes/` mirror (where "you" is Bob's petname for you)
- Charlie's devices receive it in their `you/notes/` mirror
- Both can update the document
- Updates sync back to your mirror outposts of them

## Contact Export and Bulk Sharing

### The Problem

When you build up a contact list (say, 15 family members), it's tedious for
everyone to manually exchange contact information with everyone else. You'd need
15 × 14 = 210 manual exchanges.

### The Solution

You can export your contact list and share it with others. When you export a
user's contact information, you send:

- Their nickname
- Their master identity key (public key)
- All their device records with signatures
- Optional: suggested petname

Recipients can:

- Import the identity key and devices
- Verify all signatures cryptographically
- Choose their own petname (or use your suggestion)
- Trust the identity because the signatures are self-verifying

This works because the identity is cryptographically bound. You're not asking
recipients to trust you; you're giving them the user's self-signed identity that
they can verify independently.

### Example: Family Contact List

You export contacts for your entire family. Your sister imports them:

- She gets everyone's identity keys and devices
- She assigns her own petnames ("mom", "dad", "brother", etc.)
- She can immediately sync with any of their devices
- The signatures prove the devices belong to who they claim to be

## Document Links

### Linking by UUID

Documents are linked by their UUID, not by filename or path. This means:

- Links remain stable even when files are renamed or moved
- Different users can organize files differently
- Links work across mirror outposts

When you write `[[dinner-story]]` in your markdown, Fieldnote resolves this to a
UUID internally. When Joe's mother writes `[[that-fun-evening]]` referring to
the same document, it's also resolved to the same UUID. Both links point to the
same shared document regardless of what each person calls it or where they store
it.

### User References

When linking to users in documents, you reference them by petname locally, but
this is backed by their cryptographic identity. If multiple people reference the
same user by different petnames, they're all referring to the same identity key.

## Identity Rotation and Recovery

### Lost Master Key (Manual Recovery)

If you lose your primary device and didn't back up your master key:

1. Generate a new master identity key on a new device
2. Re-sign all your devices with the new key
3. Export your new identity
4. Share it out-of-band with your peers (text message, email, in person)
5. Peers manually update their records: "new key XYZ is alice"

### What Continues to Work

After manual key rotation:

- Document UUIDs remain stable
- Links in documents still resolve correctly
- Your mirror outposts on peer systems remain intact
- Peers can associate old documents with your new identity

The manual out-of-band verification step is essential - there's no way around
peers needing to confirm "this new key is actually Alice" through some trusted
channel. This is acceptable for small groups where key loss is rare.

## Security Properties

### What's Verified

- Device signatures prove a device belongs to a user
- Identity keys are cryptographically unique
- No one can impersonate a user without their private key
- Signature chains are verifiable by anyone

### What's Trusted

- Out-of-band contact exchange (first meeting or export)
- Manual key rotation verification (when master key is lost)
- The directory-as-permission-boundary model

### What's Not Tracked

- Individual document ownership (the outpost owns all its documents)
- Per-document permission levels (sharing is all-or-nothing per user)
- Edit history provenance (LWW with vector timestamps, no signatures per edit)

This design prioritizes simplicity and human usability over perfect
cryptographic provenance. For a personal note-taking system shared among trusted
contacts, this tradeoff is appropriate.
