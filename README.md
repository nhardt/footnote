# Fieldnote

Fieldnote is a command line, p2p sync and share engine for markdown documents
with yaml frontmatter.

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

A document checkpoint is: `{uuid}_{vector_timestamp}_{path}`

UUID uniquely identifies the document. vector_timestamp is the modified date.
Fieldnote will initially implement LWW with a vector timestamp. The path is
included for file moves.

#### Users

Users are added locally using the name they are to you. You also add their
devices by hand, or from a trusted peer, or from an identity export.

Details: [docs/users.md]

#### Devices

A device has an iroh Endpoint address and a name. The name is not functional and
should have a name like "laptop", "iphone", "desktop", etc.

Every device can read, create and update notes at any time but changes are just LWW.

##### Primary Device

Each user has a single primary device. Internal to the user, all devices will
sync to primary. The user manages this. If two devices attempt to sync and both
are marked as primary, sync'ing will be blocked and the user will be notified.

A primary device handles two things uniquely: Remote device sync and file
deletes. If I, as a user, add multiple devices for a remote user, I do not
need to manage those as being primary. User-to-User sync via primary is just
an optimization.

## Syncing

Syncing occurs between all devices owned by a user. When a device thinks it has data
for a peer, it attempt to connect. On connect, it sends a manifest. The acceptor
will check see if all note chekckpoints match. If the connector has a newer write,
the acceptor will request it. The acceptor will also store the connector's manifest.

## Sharing

Sharing is user to user sync. The model allows sharing selected documents with others
for shared research while retaining ownership.

### Disk Layout

The layout on Alice's fileystem:

- devices
  - phone.md
  - laptop.md
  - desktop.md
- notes
  - that_one_time.md
  - interesting_new_things.md
- outposts
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
in outpost/bob/notes/. devices associated with bob, within reason, should be
considered to have a lease on that subdirectory. when a device of bob's
connects, it will send a manifest of what should be in that directory, which
will be a mirror of all files that bob has shared with alice. alice's device
will then connect back to bob to request files.

### Device File Layout

```markdown
---
iroh_endpoint_id: asdfasfasdfsadf
---

Mardown text here at user's discretion
```

### Note File Layout

```markdown
---
uuid: 12312321-sdff1-234fd-234-231
share_with:
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

This will create the on-disk directory structure, and add the local device as a primary device.

#### Add a second device

Adding a second device requires the primary and secondary device to connect.

On the primary device:
```
$ fieldnote device create

Listening for new device...
Copy this to your new device:
  iroh://abc123def456...?token=xyz789
```

On the secondary device:
```
$ fieldnote device create remote "iroh://abc123...?token=xyz789" --device-name "my-phone"

Connecting to primary device...
Authenticating...
Receiving identity...
- [x] Joined as device 'my-phone'
- [x] Identity: @alice-jones (master key: def456...)
```

### User

Fieldnote allows a user to manage all these objects via a cli tool.

```
fieldnote user create {user_name}
fieldnote user delete {user_name}

fieldnote device create
fieldnote device create remote {remote_url} --device-name {device_name}
fieldnote device delete {user_name} {device_name}

fieldnote mirror listen
fieldnote mirror push --user {user_name} --device {device_name}
```

## Future Improvements

- [ ] CRDTs for internal shared writes
- [ ] A more advanced permissions system would be cool
- [ ] Shared ownership
- [ ] An actual distributed permission system

## References

- https://iroh.computer/
- https://files.spritely.institute/papers/petnames.html
