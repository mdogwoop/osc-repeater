# osc-repeater

The `osc-repeater` listens on an arbitrary number of ports, combines all
messages from them into a common stream.
Anything in that common stream is then sent on to an arbitrary number of target
OSC servers.

## Building

This project is written in Rust. To build:

```bash
cargo build --release
```

The binaries will be available in `target/release/`:
- `osc-repeater`: The main repeater application
- `osc-send`: A utility for sending test OSC messages

## Usage

### Running the repeater

```bash
cargo run --bin osc-repeater -- -c config.yaml
```

Or with the compiled binary:

```bash
./target/release/osc-repeater -c config.yaml
```

Add the `--debug` flag for debug logging:

```bash
cargo run --bin osc-repeater -- -c config.yaml --debug
```

### Sending test messages

```bash
cargo run --bin osc-send -- -p 8080
```

Or with the compiled binary:

```bash
./target/release/osc-send -p 8080
```

## Configuration

Create a `config.yaml` file with the following structure:

```yaml
listenPorts:
  - 8080
  - 8081
targets:
  - "192.168.1.1:2597"
  - "192.168.1.2:2598"
```

- `listenPorts`: List of ports to listen on for incoming OSC messages
- `targets`: List of target addresses (IP:port) to forward messages to

