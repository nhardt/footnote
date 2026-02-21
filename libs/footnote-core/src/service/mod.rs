pub mod join_service;
pub mod sync_service;

// protocol negotation is a more robust solution to wire changes. bumping the
// version on a compatability change is probably the bare minimum
//
// v(-1)
// file exchange protocol:
// - on push from device A to device B
//
// A creates manifest for B
// A sends manifest to B
// B reads manifest, looks for needed files
// B requests files from A
// A verifies B can read requested file
// A sends file to B
//
// vNone:
// sender sends user record manifest
// receiver writes user record, if successor
// receiver compares manifest to local files
// receiver requests files from manifest
// sender validates file is from manifest, sends
//const ALPN_SYNC: &[u8] = b"footnote/sync";
//
// v1:
// sender sends user record, serialized contact array, manifest
// receiver writes user record, if successor, contact array, if coming from device_leader
// receiver compares manifest to local files
// receiver requests files from manifest
// sender validates file is from manifest, sends
//
// v2:
// sender sends user record, serialized contact array, manifest, tombstone
// receiver writes user record, if successor, contact array, if coming from device_leader
// receiver compares manifest to local files
// receiver requests files from manifest
// sender validates file is from manifest, sends
// receiver deletes files in the tombstone, if deleted_time < modified_time

pub const ALPN_SYNC: &[u8] = b"footnote/sync/2";
