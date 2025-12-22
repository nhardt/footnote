# Footnote CLI

The footnote CLI is a command line interface to the footnote sync and share
mechanisms.

## Objects

### Vault

Vault is the primary concept in Footnote. It creates Manifests to determine
which Devices of your Contact's should have which Notes.

### Device

Devices are entities capable of storing files and talking over a network,
generally hardware but virutal hardware works just fine.

#### Primary Device

One device is designated as primary for a user. That device mints the user's
contact record, which has a list of their devices, a vector timestamp and is
signed with the user's public key.

### Contact

A contact is person. A contact owns one or more devices. Your own devices are
stored in a special contact record.

### Note

A note in Footnote is a title, yaml frontmatter, text/markdown body, markdown
compatible footnotes. Inside the yaml frontmatter is a uuid and vector
timestamp. Files can be anywhere in your vault, the primary record information is
the uuid+timestamp.

## Actions

### Vault Create

This is the first thing a user does to create a vault on their primary device.
Vault Create will create a .footnote directory, signifying the directory can be
used with footnote. Initial records will be written, including the user's
primary public/private keypair, the iroh device id and a contact record
specifying this information.

### Device Create/Vault Join

Joining a Vault is a one time act a user performs on their devices. It's a two
way handshake initiated on the primary device. A url string must be transferred
OOB from the primary device to the joining device. The primary device will create
the url, print it for the user, then listen.

The user transfers the url from the primary to the secondary device. On the
secondary device, the user will call Vault Join, providing the connection
string. The secondary device contacts the primary, the primary mints a new
contact record and sends it to the secondary.

### Vault Listen (Files)

When a vault on a device is listening, it will post to Iroh's public server
network that it is online. As part of the technology, when a device connects,
it's public key, encoded in the iroh endoit id, is intrisic to the connection.
If a device connects that is not known, the connection is immediately dropped.
If the connection is from a known device, and the endpoint is from our own
device, a Sync is performed.

### Sync

Internal to Footnote, file tranfers between devices owned by the same users is
called a Sync. Sync ultimately wants all devices belong to a user to be
consistent, including your notes and notes that have been shared with you.

### Contact Export

Once a user has their devices setup, they will be able to set up a share
relationship with a known contact. The do this by exporting their own contact
record. The contact record is then sent OOB to a known, trusted sharing partner.
The footnote protocol is currently designed just for this type of sharing
relationship.

### Contact Import

The inverse of Import. To establish a share relationship, your partner will
send you a contact record. The record contains a public key, iroh endpoints
and devices.

### Share (User)

Sharing is the act of transferring files from a user to the users that they have
shared files with. This will generally be a primary-to-primary connection, but
not required to be. The device that connects will attempt to make their footnote
directory as up to date as they are able.

## Details

### Moving a primary device

A contact record has a vector timestamp and public key. A contact record
transferred over a validated connection signed by a public key can reasonably be
trusted to be an update to an existing contact record, and can happen
internally.

### Losing your primary device

If a primary device is lost, you will need to create a new contact record. The
contact record can be sent to users you are already paired with. The user
receiving the record will overwrite their record with the given petname with the
new record.
