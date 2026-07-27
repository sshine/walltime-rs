## CLI for measuring time spent in a process.

`wtime` is a modern replacement for the UNIX `time` command. It runs a
command, streams its output (optionally with timestamps), tracks phases
via regex matching, and prints a colorful timing summary with run history
comparison.

## Installation

To run `wtime` without installing it:

```
$ nix run github:sshine/walltime-rs -- cargo build
```

To install it, apply the overlay and take the package from `pkgs`. It is
exposed as both `walltime-cli` (the crate) and `wtime` (the binary):

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    walltime-rs = {
      url = "github:sshine/walltime-rs";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, walltime-rs, ... }:
    {
      nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          (
            { pkgs, ... }:
            {
              nixpkgs.overlays = [ walltime-rs.overlays.default ];
              environment.systemPackages = [ pkgs.wtime ];
            }
          )
        ];
      };
    };
}
```

The overlay builds against whichever nixpkgs it is applied to, so the package
can be overridden and cross-compiled like any other.

## Examples

### Default

Without any parameters, `wtime` produces a summary with history at the end:

```
$ wtime cargo build
   Compiling proc-macro2 v1.0.106
   ...
   Compiling walltime-cli v0.1.0
    Finished `dev` profile in 7.31s

──────────────────────────────────────────────────
  walltime summary
──────────────────────────────────────────────────
  Total:      7.329s

  History: cargo build (last 6 runs)
  ┌─────┬──────────────┬────────┬──────┐
  │ Run │         Date │  Total │ Exit │
  ├─────┼──────────────┼────────┼──────┤
  │  #1 │ Apr 06 14:42 │ 7.615s │    0 │
  ...
  │  #6 │ Apr 06 14:46 │ 7.328s │    0 │ ← current
  └─────┴──────────────┴────────┴──────┘

  Exit code:  0
──────────────────────────────────────────────────
```

Only successful runs (exit code 0) are included by default.

When the program fails it prints all the failed runs instead.

Use `wtime -a ...` to include all runs regardless of exit code.

### Timestamps

To add wall-clock timestamps to every output line:

```
$ wtime -t cargo build
[14:47:46.143]    Compiling proc-macro2 v1.0.106
[14:47:46.143]    Compiling unicode-ident v1.0.24
...
[14:47:52.923]     Finished `dev` profile in 6.85s
...
```

Timestamp format can be modified with `-f <format>`, e.g. UNIX epoch with milliseconds:

```
$ wtime -t -0 -f '%s%.3f' cargo build
[946684800.090]    Compiling proc-macro2 v1.0.106
...
[946684801.881]    Compiling errno v0.3.14
...
[946684807.345]     Finished `dev` profile in 7.33s
...
```

### Zero timestamps

To add timestamps that begin from zero instead of current local time:

```
$ wtime -t -0 cargo build
[00:00:00.086]    Compiling proc-macro2 v1.0.106
[00:00:00.086]    Compiling unicode-ident v1.0.24
...
[00:00:07.018]     Finished `dev` profile in 7.01s
...
```

### Dynamic phase-tracking

To track phases of a process, use `-d` and a regex with a capture group:

```
$ wtime -t -0 -d 'Compiling ([^ ]*)' cargo build
...

  Phases:
    proc-macro2           0.000s  (0.0%)
    ...
    regex-automata        1.119s  (15.7%)
    ...
    chrono                0.538s  (7.5%)
    walltime-core         0.480s  (6.7%)
    walltime-cli          0.500s  (7.0%)
...
```

This helps track what part of a very verbose output is taking a long time.

Before matching the regex, all ANSI codes are stripped from the string.

To hide phases that take shorter than some amount of seconds, use `-m N` to focus on the slow ones:

```
$ wtime -t -0 -d 'Compiling ([^ ]*)' -m 1 cargo build
...

  Phases:
    ctrlc                 1.460s  (19.7%)
    clap                  1.017s  (13.7%)
    (57 phases < 1.000s)  4.838s  (65.4%)  0.085s avg
...
```

Using `-m N` only omits phases when printing, all phases are still measured and saved.

### Full help output

```
A modern replacement for the UNIX `time` command.

Runs a command and provides a colorful timing summary, optional line timestamps, phase tracking, and run history comparison.

Usage: wtime [OPTIONS] <COMMAND>...

Arguments:
  <COMMAND>...
          Command to run

Options:
  -t, --timestamps
          Enable line timestamp prefixing

  -f, --timestamp-format <TIMESTAMP_FORMAT>
          Timestamp format (chrono syntax)

          [default: %H:%M:%S%.3f]

  -0, --from-zero
          Count timestamps from 00:00:00.000 instead of wall-clock time

  -p, --phase <NAME=REGEX>
          Define a phase boundary (repeatable). Format: NAME=REGEX

  -d, --dynamic-phase <REGEX>
          Regex with a capture group for dynamic phase names (repeatable)

  -m, --min-phase <SECONDS>
          Hide phases shorter than this threshold in the summary (seconds)

          [default: 0]

      --no-summary
          Suppress the timing summary

      --no-log
          Don't save or show the run log

      --log-file <LOG_FILE>
          Log file path

          [default: .walltime.jsonl]

  -a, --show-all
          Show all runs in history (by default, only matching-outcome runs are shown)

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
