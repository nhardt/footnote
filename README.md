## iroh-cli

Minimal code to test an iroh connection.

Much better code examples:

- https://github.com/n0-computer/iroh-doctor/blob/main/src/commands/accept.rs
- https://github.com/n0-computer/iroh-doctor/blob/main/src/commands/connect.rs
- https://github.com/n0-computer/iroh/blob/dd99737c12c553ece2607e5e74d605751a637397/iroh/src/endpoint.rs#L2550

Much better testing tool:

https://github.com/n0-computer/iroh-doctor/

## Usage

### Connect/Ping

#### Two devices

If you're running a test from two computers, you can use the default keys.

Computer 1:

> cargo endpoint create
> cargo endpoint read

(get the endpoint id from computer 1 to computer 2 however is most convenient)

> cargo ping listen

Computer 2:

> cargo endpoint create
> cargo ping connect {computer1_endpoint_id}

Your devices are now free to move about the internet

#### One device

For less difficult round trip testing, the commands optionally take a key name.

Terminal 1:

> cargo endpoint create ep1
> cargo endpoint read ep1

(copy the end point id)

> cargo ping listen ep1

Terminal 2:

> cargo endpoint create ep2
> cargo ping connect ep2 {paste ep1 connection string}

### Sync

Sync is a simple file mirroring protocol. When iroh-cli is running in sync
listen mode, it will listen for connections. If it receives a connection for an
endpoint for which it has a data directory, it will accept a manifest for that
directory. The manifest will be capped at 1MiB and will represent the files that
the connecting endpoint wants to mirror there.

#### Setup

##### Single Device

We'll pretend to be two devices here. First create both devices, represented as unique endpoints.

> cargo endpoint create dev1
> cargo endpoint create dev2

Now create on-disk structures to simulate the two devices. Each device will have
an imagined directory they want to mirror to the remote

> mkdir -p ./data/dev1/mirror_to/dev2/
> mkdir -p ./data/dev1/mirror_for/dev2/
> echo "# The best basketball shot I ever made" > ./data/dev1/mirror_for/dev2/escape_from_la.md
> mkdir -p ./data/dev2/mirror_to/dev1/
> mkdir -p ./data/dev2/mirror_for/dev1/
> echo "# A story about how my life got turned upside down" > ./data/dev2/mirror_for/dev1/belaire.md

We now have two theoretical devices, each with a file they want to sync to the
other. Conceptually, dev1 owns the directory mirror_to/dev2, and any files there
are intended to be mirrored on dev2 in the mirror_from_dev1 directory.
Obviously, this is a trust situation. dev2 and dev1 should be owned by two people
that know each other. Putting in reasonable safeguards for the max size of the
directory will not be included in the example.

Now, start one instance of the sync engine as dev1, and one as dev2

Terminal 1:

> cargo run sync listen dev1

Terminal 2:

> cargo run sync push dev2 dev1

In a different terminal

> cargo run sync_watch_as dev2

At this point, dev1 and dev2 will use a shared .keys directory to map a name to
device.

## References

- https://iroh.computer/
- https://github.com/n0-computer
