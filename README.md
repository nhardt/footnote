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

## References

- https://iroh.computer/
- https://github.com/n0-computer
