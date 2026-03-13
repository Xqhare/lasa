# Lasa

Lasa is a simple rust program to measure and log the system uptime.

## Features

- Measure and log the system uptime.
- Writes statistics in both `.xff` and `.json` files.
  - `.xff`: A single binary XFF file, perfect for programmatic access.
  - `.json`: A single, prettyfied, JSON file, perfect for human consumption.
- Only needs to run at startup.
- Uses the `Batch` scheduler policy and a nice value of 19.
  - This means that the program will only run during low system utilization.
- System crash aware
  - Finds the last log before a system crash and uses that as the shutdown time.

## What data is saved?

All data collected is stored locally in the directory of the program.
Statistics are stored in the `Home` directory.

### Provided Statistics

In both the `.xff` and `.json` files, the following statistics are provided:

- `all_time`: The total uptime of the system.
  - **uptime_percent**: The percentage of time the system has been running.
  - **total_downtime_seconds**: The total number of seconds the system has been down.
- `current_year`: The current year.
  - **year**: The current year.
  - **uptime_percent**: The percentage of time the system has been running in the current year.
- `current_month`: The current month.
  - **month**: The current month.
  - **uptime_percent**: The percentage of time the system has been running in the current month.

## Usage

Run the program at startup.

Read the generated `.xff` and `.json` files for statistics.

## Naming

Lasa is the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

For more information about the design and history of this repo [click here](./historical-startup-notes.md).
