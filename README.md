# Gakun (Rust Version)

Gakun is an SSH key manager that allows you to manage and switch your SSH keys per host easily. Unlike [skm](https://github.com/TimothyYe/skm), which manages keys in separate directories and utilizes symbolic links, Gakun manages SSH keys directly within the `~/.ssh` directory, as it should be. This approach ensures that whenever you want to return to manual management, you will still have great control over your keys.

This is a Rust rewrite of the original Go implementation with the same behavior and logic.

This software is still in heavy development. Please expect breaking changes and use it at your own risk.

## Building

```bash
cargo build --release
```

The binary will be available at `target/release/gakun`.

## Installation

```bash
cargo install --path .
```

Or copy the binary to your PATH:

```bash
cp target/release/gakun /usr/local/bin/
```

## Usage

```
SSH key manager

Usage: gakun <COMMAND>

Commands:
  add      Add host and key to a profile. Example: 'gakun add work gitlab.com ~/.ssh/id_rsa_work'
  use      Use SSH key for certain host. Example: 'gakun use work -h gitlab.com'
  ls       List profiles
  detach   Detach gakun - remove gakun-managed section from ~/.ssh/config
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

### Commands

#### Add a profile

Add a host and SSH key to a profile:

```bash
gakun add work -h gitlab.com -k ~/.ssh/id_rsa_work
```

#### Use a profile

Switch to use a specific SSH key for a host:

```bash
gakun use work -h gitlab.com
```

This will update your `~/.ssh/config` file to use the specified key for the host.

#### List profiles

View all configured profiles:

```bash
gakun ls
```

#### Detach gakun

Remove the gakun-managed section from your `~/.ssh/config` file, returning it to manual management:

```bash
gakun detach
```

You can also use the short alias:

```bash
gakun d
```

This command will cleanly remove the gakun-managed section (the lines between `###### gakun begin` and `###### gakun end`) from your SSH config, leaving the rest of your configuration intact.

## How it works

Gakun stores profile configurations in `~/.config/gakun/config.json`. When you "use" a profile for a host, it updates your `~/.ssh/config` file by adding a managed section at the top:

```
###### gakun begin
Host gitlab.com
  Hostname gitlab.com
  IdentityFile /path/to/your/key
###### gakun end
```

Any existing gakun-managed section is automatically replaced, while the rest of your SSH config remains untouched.

## License

MIT
