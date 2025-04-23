# Fedimint DKG CLI

A command-line tool to help Fedimint guardians complete the Distributed Key Generation (DKG) process without using the admin UI.

Note: a similar process can also be done using `fedimint-cli admin setup`.

## Features

- Check the status of the setup process
- Set local parameters for a guardian (name and optional federation name)
- Add peer connection information
- Start the DKG process
- Reset peer setup codes

## Installation

### From Source

```bash
git clone https://github.com/douglaz/fedimint-dkg-cli-standalone.git
cd fedimint-dkg-cli-standalone
cargo build --release
```

The binary will be available at `target/release/fedimint-dkg-cli`.

## Usage

### Global Options

- `--password <PASSWORD>`: Password for API authentication (required for all commands)

### Commands

#### Check Status

```bash
fedimint-dkg-cli --password <PASSWORD> status --api-url <API_URL>
```

#### Set Local Parameters

```bash
fedimint-dkg-cli --password <PASSWORD> set-local-params --api-url <API_URL> --name <NAME> [--federation-name <FEDERATION_NAME>]
```

#### Add Peer

```bash
fedimint-dkg-cli --password <PASSWORD> add-peer --api-url <API_URL> --peer-info <PEER_INFO>
```

#### Start DKG

```bash
fedimint-dkg-cli --password <PASSWORD> start-dkg --api-url <API_URL>
```

#### Reset Peer Setup Codes

```bash
fedimint-dkg-cli --password <PASSWORD> reset-peer-setup-codes --api-url <API_URL>
```

## Bulk Setup Script

Run the included `simple_dkg.sh` helper to distribute setup and start DKG across all peers.

```bash
$ ./simple_dkg.sh -h
Usage: simple_dkg.sh [-c CLI_PATH] [-p PASSWORD] [-s SUFFIX] [-n FED_NAME] [-h|--help]

Options:
  -c CLI_PATH    Path to fedimint-dkg-cli binary (default: ./target/release/fedimint-dkg-cli)
  -p PASSWORD    API password (default: testpass)
  -s SUFFIX      API suffix
  -n FED_NAME    Federation name (default: test federation)
  -h, --help     Show this help message
```

## Example

Setting up a 4-guardian federation:

1. Guardian 1 (Leader):
   ```bash
   fedimint-dkg-cli --password pass set-local-params --api-url wss://api.alpha.example.com/ --name alpha --federation-name test-federation
   ```

2. Guardian 2:
   ```bash
   fedimint-dkg-cli --password pass set-local-params --api-url wss://api.bravo.example.com/ --name bravo
   ```

3. Guardian 3:
   ```bash
   fedimint-dkg-cli --password pass set-local-params --api-url wss://api.charlie.example.com/ --name charlie
   ```

4. Guardian 4:
   ```bash
   fedimint-dkg-cli --password pass set-local-params --api-url wss://api.delta.example.com/ --name delta
   ```

5. Exchange connection codes between guardians and add peers:
   ```bash
   # On Guardian 1
   fedimint-dkg-cli --password pass add-peer --api-url wss://api.alpha.example.com/ --peer-info "connection-code-from-bravo"
   fedimint-dkg-cli --password pass add-peer --api-url wss://api.alpha.example.com/ --peer-info "connection-code-from-charlie"
   fedimint-dkg-cli --password pass add-peer --api-url wss://api.alpha.example.com/ --peer-info "connection-code-from-delta"
   
   # On Guardian 2
   fedimint-dkg-cli --password pass add-peer --api-url wss://api.bravo.example.com/ --peer-info "connection-code-from-alpha"
   # ... and so on for all guardians
   ```

6. Start DKG on all guardians:
   ```bash
   # On each guardian
   fedimint-dkg-cli --password pass start-dkg --api-url wss://api.guardian.example.com/
   ```

## License

This project is licensed under the [MIT License](LICENSE).
