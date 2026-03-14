# Lasa

Lasa is a simple rust program to measure and log the system uptime. 

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
  - Adheres strictly to the Pantheon ecosystem rules, relying only on standard libraries and internal crates like `athena`, `horae`, and `nabu`.

## What data is saved?

The primary database is stored in the user's **Cache Directory** for fast, ephemeral access without unnecessary disk I/O.
The final statistics are stored in the user's **Home** directory.

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

Simply run the program at startup.

```bash
lasa
```

Read the generated `.xff` and `.json` files in your home directory for statistics.

## Naming

Lasa is the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

For more information about the design and history of this repo [click here](./historical-startup-notes.md).
