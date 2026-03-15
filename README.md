# Lasa

Lasa is a simple, unix only, rust program to measure and log the system uptime. 

In contrast to many other system monitors, Lasa is a "run once" program. This means that all calculations are done once and the program exits.
No background processes are kept running.

Lasa, as all my projects, is written on top of only the rust standard library, `libc` and crates written by me using the same policy.
It does call two system calls, `last` (from `util-linux`) and `journalctl` (from `systemd`), but these are the only external "dependencies".

## Features

- Measure and log the system uptime.
- Writes statistics in both `.xff` and `.json` files.
  - `.xff`: A single binary XFF file, perfect for programmatic access (e.g. status bars).
  - `.json`: A single, prettified JSON file, perfect for human consumption.
- Only needs to run at startup.
- Uses the `Batch` scheduler policy and a nice value of 19.
  - This means that the program will only run during low system utilization, being a "quiet attendant" to your system.
- System crash aware
  - Finds the last log before a system crash by probing `journalctl` and uses that as the shutdown time to ensure accuracy.
- Zero External Dependencies
  - Relying only on standard libraries and internal crates like `athena`, `horae`, and `nabu`.

## What data is saved?

The primary database is stored in the user's **Cache Directory** for fast, ephemeral access without unnecessary disk I/O.
The final statistics are stored in the user's **Home** directory.

- **Database**: `~/.cache/lasa_system_uptime_db.xff`
- **Statistics**: `~/lasa_system_uptime.json` & `~/lasa_system_uptime.xff`

### Provided Statistics

In both the `.xff` and `.json` files, the following statistics are provided:

```json
{
    "all_time": {
        "total_downtime_seconds": 45120.0,
        "uptime_percent": 98.42
    },
    "current_month": {
        "month": 3,
        "total_downtime_seconds": 944.0,
        "uptime_percent": 97.5
    },
    "current_year": {
        "total_downtime_seconds": 45120.0,
        "uptime_percent": 99.1,
        "year": 2026
    }
}
```

## Usage

Lasa provides no precompiled binaries.

To build the program, please first make sure you have [Rust](https://www.rust-lang.org/) installed.

Then, clone the repo and run `cargo build --release` to build the binary.

> **Note:** The repository uses a customised build profile to keep the size of the binary as small as possible.
> This means that your build times will be some what longer than usual.

The generated binary can be found inside the `/target/release` directory.

Move it to a location of your choice, such as `/usr/local/bin`.

Simply run the program at startup automatically or manually.

Lasa also provides a systemd service file, which you can find in the root of the repo.
If you need more information on what this file does, please see the [systemd documentation](https://www.freedesktop.org/software/systemd/man/systemd.service.html).

Read the generated `.xff` and `.json` files in your home directory for statistics.

## Naming

Lasa is the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

## License

MIT License

---

For more information about the design and history of this repo [click here](./historical-startup-notes.md).
