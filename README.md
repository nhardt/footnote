# Footnote

Footnote is a local-first personal wiki of markdown documents with support for
mirror across your own devices and sharing with known, trusted peers.

## Core Concepts

### Personal Wiki

Your notes are a markdown wiki. Notes live in a vault and only ever exist on
your devices. They are always encrypted in transit over the network, and
only transmitted to known devices and known peers.

### Vault

A vault is folder on a filesystem that represents your own notes and the notes
that have been shared with you.

### Trusted Sources

Your vault has a special directory called `footnotes`. Inside `footnotes` are a
list of directories, one per trusted contact, listed under the name you want to
call that person. Trust is established with a one-time, out-of-band setup
process. Each document has an owner, the owner updates them as neded and sends
them to you when there are changes.

An example of something you might have in your vault is: `footnotes/mom/recipes.md`

### Share vs Mirror

"Share" is the exchange of documents among trusted peers. The view of your
documents is unique to each person you share with.

"Mirror" is defined as coordinating an eventually consistent view of all your
notes across your owned devices, including things shared with you. The goal of a
mirror is that you can update the log of your running activities while you're
out and about, and process them on your desktop when you home later.

### Infrastructure

#### Public

The internet infrastucture required to facilitate data exchanges are Iroh
signaling servers. An end user just needs a device that is on somewhere.

#### Device

Each user can have multiple devices (laptop, phone, tablet, desktop). Every device:

- Has its own Iroh endpoint ID for network connectivity
- Is cryptographically signed by the user's master identity key
- Has a human-friendly name (e.g., "laptop", "phone")
- Can be verified as belonging to that user

#### Device Group

A collection of devices owned by a user. Devices in a device group intend to
mirror each other.

#### Device Group Leader

Only one of the devices maintains the list of devices in the group. The leader
is where you make changes, the leader distributes the list of members to other
members in the group. The leader device can transfer leadership to a different
device.

#### Often-on Device

The device group works best if one of the devices can be online somewhere. It's
not required, but having somewhere to back your notes when you're out and about
is nice.

#### Group Types

##### Mobile only

This works fine. The device will only attempt to sync when it's online and the
app is in use.

##### Mobile/Desktop

This works fine. The mobile device will only attempt to sync when it's online.
The desktop device will be online as much as you want it to be. If it's always
on, every mobile edit will replicated off the mobile device in short order. As
a user, if you're working on your phone and want to start working on your desktop,
it's easy to ensure your most recent work gets sync'd to your desktop by opening
the app in both places.

In all setups, if you have a mobile device it works best as your device leader.
It has the most mechanisms to connect (nearby share, air drop, qr code).
Generally, it's worst as an "often on" device as mobile devices tend to
prioritize battery usage over everything. Still, phones are able to notify you
of new email, new texts, etc, so it's probably possible to make this efficient
on mobile as well.

##### Mobile/Desktop/Laptop/MiniPC

From here the cases are mostly the same. Mobile for device list management,
some other device for "often on" connectivity.

Sharing between users favors the often-on devices.

### Identity

Identity is created on an as-needed basis. When you create a vault, it is in
stand alone mode. You are free to create and link notes and research without
ever making an identity. To mirror these notes on multiple devices, each devices
get an identity and name. Devices are joined together in a device group
manually.

To trade notes with another person, a public key identity and contact record
will be created. The contact record contains all devices the device group, the
group leader and some other metadata. Contact records can be shared between
group leaders via a few in band and many out OOB mechanisms.

The records are verified by signing key fields into a digitial signature:

- Username
- Public Key
- Devices
  - Iroh Endpoint Id
  - Name

After contact records are exchanged, each user has a spot in their notes to
reference their peers' notes. When the contact is imported, the importing user
decides what to call that person. That nickname their directory name in
`footnotes/`. If you import Mom's contact record and use "mom" for her nickname,
the directory her shared files will be in is footnotes/mom/.

#### Security Considerations

- Records are verifably self consistent
- Public key is not identity, but two records signed with the same key are
  expected to be created by the same secret key.
- The contact record exchange mechanism is a potential weak point.
- Iroh protocol identifiers are sent in plain text.
- The overall design goal is data ownership, vendor neutrality, no data mining
  for marketing, etc. The app is not anonymous, you should know your trusted
  contacts as you will be allowing them to write files to your hard drive.

#### Attack Vector Considerations

##### Replace Iroh Endpoint Id with false endpoint id in contact record

Key signature fails

##### Intercept and replace entire record during exchange

Alice creates and exports record:

- Username: Alice
- Public Key: aaa-key-aaa
- Device address: aaa-device-aaa

Charlie creates and exports record:

- Username: Charlie 
- Public Key: ccc-key-ccc
- Device address: ccc-device-ccc

Bob wants to sit in the middle of these two. Bob creates

Bob-Alice:

- Username: Alice
- Public Key: aaa-bbb-key-bbb-aaa
- Device Address: aaa-bbb-device-bbb-aaa

Bob-Charlie:

- Username: Charlie 
- Public Key: ccc-bbb-key-bbb-ccc
- Device Address: ccc-bbb-device-bbb-ccc

This does seem possible. The mechanism by which you transfer contact records
would have to be compromised. You could add additional layer of verification by
posting your public key somewhere, or calling up your peer to spot check key
fields.

##### Modify record after exchange

Key signature fails

##### Replace endpoint

Alice and Charlie are legitimately connected. Bob wants to listen at
ccc-key-ccc. To do this, Bob would need ccc-key-ccc. This is true of any
public/private key pair. If this occurs because Charlie's machine is
compromised, no deeper level of access is granted.

##### File leakage - Mirror

To accidentally share your full vault with an unintended device, the device's
iroh endpoint id would need to be signed in to your user record, and would need
to pass verificiation. Internal to the software, the pairing occurs with a
one-time pairing code. For an incorrect device to be there unintentionally, a
user would need to set up their device to listen, then send the one time code to
a peer.

For a 3rd party to insert a false record, they would need hard drive access,
which is also where all the files are. The record would appear in the UI.

##### File leakage - Share

The sharing protocol:
- Alice wants to share with Charlie
- Alice tags files with here nickname for Charlie (Chuck)
- Alice connects to Chuck, send list of available files
- Charlie calls back and asks for advertised file
  - Alice's device checks each contact file for the iroh endpoint id
  - Alice's device gets the nickname for the associated user
  - Alice's device verifies the contact record
  - Alice's device verifies the nickname is in the file
  - Alice's device sends the file

The mechanism relies on the contact exchange and iroh's endpoint verifications.

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

I found Alice's research[1] really insightful.

This connects to my earlier thoughts in [[interesting_ideas]].

[1]: footnote.wiki://450332400-e29b-41d4-a716-446655440000

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
- debt: rework components to take the data they need, vs taking a path and getting it
- debt: factor tailwind heavy primitives (button, modal)
- Trigger mirror immediately on save

### Medium

- contact refresh
- local file rename 
- local file deletes
- Contact distribution upon update
- sync log
- directory level sharing

### Big

- replicate file deletes (probably can get by with a path, deleted timestamp)
- drag/drop sharing: in contact_view, ability to include/exclude files
- groups for sharing
- automated testing across supported platforms
- scale testing (targeting 200 peers max)

## Under Consideration (how/if/when)

- "as if" view. browser your files as if you are a user you share with
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
- windows

## References

- https://www.inkandswitch.com/keyhive/notebook/
- https://files.spritely.institute/papers/petnames.html
