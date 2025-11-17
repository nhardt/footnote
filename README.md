# Fieldnote

Fieldnote is a command line, p2p sync and share engine.

## Basics

Fieldnote is intended to sync all your device notes to a common, primary device,
a laptop or desktop. A nuc in your house. Fieldnote can also sync to peers that
you have manually added. A good setup would be if you and friend connect your
primary, usually-on devices.

### Nouns

#### Notes

Notes are markdown files, with yaml frontmatter for metadata. Every note has a
uuid, which is its canonical identifier.

##### Note Checkpoint

A document checkpoint is: `{uuid}-{vector_timestamp}-{path}`

UUID uniquely identifies the document. vector_timestamp is the modified date.
Fieldnote will initially implement LWW with a vector timestamp. The path is
included for file moves.

#### Users

Users are added locally using the name they are to you. You also add their
devices by hand, or from a trusted peer.

There is a special "me" user. Any devices you add to the "me" user are sync'd
fully between each other.

Each user has a single primary device. Internal to your devices, they will all
sync to primary. The user is in charge of this. If two devices attempt to sync
and both are marked as primary, sync'ing will be blocked and the user will be
notified.

#### Devices

A device has an iroh Endpoint address and a name. The name is not functional and
should have a name like "laptop", "iphone", "desktop", etc.

Every device can read, create and update notes at any time but changes are just LWW.

##### Primary Device

A primary device handles two things uniquely: Remote device sync and file
deletes. If I, as a user, add multiple devices for a remote user, I do not
need to manage those as being primary. User-to-User sync via primary is just
an optimization.

## Syncing

Syncing occurs between any devices owned by "me". When a device thinks it has data
for a peer, it attempt to connect. On connect, it sends a manifest. The acceptor
will check see if all note chekckpoints match. If the connector has a newer write,
the acceptor will request it. The acceptor will also store the connector's manifest.

## Sharing

Sharing is user to user sync. The model allows sharing selected documents with others
for shared research while retaining ownership.

### Disk Layout

The layout on Alice's fileystem:

- me
  - devices
    - phone.md
    - laptop.md
    - desktop.md
  - notes
    - that_one_time.md
    - interesting_new_things.md
- bob
  - devices
    - desktop.md
  - notes
    - requests_for_alice.md
    - my_favorite_movies.md
- charlie
  - devices
    - desktop.md
  - notes
    - my_favorite_movies.md
    - bobs_surprise_party_details.md

The key thing to note here is that everything shared from bob to alice will be
in bob/notes/. devices associated with bob, within reason, should be considered
to have a lease on that subdirectory. when a device of bob's connects, it will
send a manifest of what should be in that directory, which will be a mirror of
all files that bob has shared with alice. alice's device will then connect back
to bob to request files.

### Device File Layout

```markdown
---
iroh-endpoint-id: asdfasfasdfsadf
---

Mardown text here at user's discretion
```

### Note File Layout

```markdown
---
uuid: 12312321-sdff1-234fd-234-231
share-with:
  - bob
  - charlie
---

Markdown text here. Primary use for app is to take notes here.
```

## Usage

### Self Setup

```
fieldnote init
```

This will create your "me" directory, generate a key pair for this device and set up your first note.

#### Add a second device

On a second device, also run `fieldnote init`.

### User

Fieldnote allows a user to manage all these objects via a cli tool.

```
fieldnote user create {user_name}
fieldnote user delete {user_name}

fieldnote device create {user_name} {device_name}
fieldnote device delete {user_name}

fieldnote sync push {optional: device id}
fieldnote share push {optional: device id}
```

## Future Improvements

- [ ] CRDTs for internal shared writes
- [ ] A more advanced permissions system would be cool
- [ ] Shared ownership
- [ ] An actual distributed permission system

## References

- https://iroh.computer/
- https://files.spritely.institute/papers/petnames.html
