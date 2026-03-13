# Lasa

Lasa is the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

Create a simple program to measure and log the system uptime.

## Second Iteration

```bash
master@serverle:~$ last reboot -F
reboot   system boot  6.12.73+deb13-am Fri Mar  6 21:00:14 2026 - still running            
reboot   system boot  6.12.73+deb13-am Fri Feb 20 05:57:35 2026 - Fri Mar  6 20:59:20 2026 (14+15:01)
reboot   system boot  6.12.73+deb13-am Fri Feb 20 02:00:52 2026 - Fri Feb 20 05:56:42 2026  (03:55)
reboot   system boot  6.12.69+deb13-am Thu Feb 12 03:00:58 2026 - Fri Feb 20 02:00:00 2026 (7+22:59)
reboot   system boot  6.12.63+deb13-am Tue Feb  3 15:05:44 2026 - Thu Feb 12 03:00:00 2026 (8+11:54)
reboot   system boot  6.12.63+deb13-am Sat Jan 31 01:52:15 2026 - Tue Feb  3 15:05:01 2026 (3+13:12)
reboot   system boot  6.12.63+deb13-am Fri Jan 30 19:06:02 2026 - Sat Jan 31 01:51:30 2026  (06:45)
reboot   system boot  6.12.63+deb13-am Fri Jan 30 19:01:37 2026 - Fri Jan 30 19:04:13 2026  (00:02)
reboot   system boot  6.12.63+deb13-am Fri Jan 30 00:02:04 2026 - Fri Jan 30 19:00:53 2026  (18:58)
reboot   system boot  6.12.63+deb13-am Thu Jan 22 19:25:19 2026 - Fri Jan 30 00:01:26 2026 (7+04:36)
reboot   system boot  6.12.63+deb13-am Wed Jan 21 22:44:40 2026 - Thu Jan 22 19:24:40 2026  (20:40)
reboot   system boot  6.12.63+deb13-am Wed Jan 21 20:56:58 2026 - Wed Jan 21 22:44:02 2026  (01:47)
reboot   system boot  6.12.63+deb13-am Tue Jan 20 20:15:52 2026 - Wed Jan 21 20:56:19 2026 (1+00:40)
reboot   system boot  6.12.63+deb13-am Tue Jan 20 16:29:37 2026 - Tue Jan 20 20:15:22 2026  (03:45)
reboot   system boot  6.12.63+deb13-am Tue Jan 20 16:11:25 2026 - Tue Jan 20 16:23:23 2026  (00:11)
reboot   system boot  6.12.63+deb13-am Tue Jan 20 16:03:52 2026 - Tue Jan 20 16:11:02 2026  (00:07)

wtmpdb begins Tue Jan 20 16:03:52 2026
```

IMPORTANT: Calculate downtime instead of uptime; meaning: This_start_time - Last_end_time

Given the above output, the entire project architecture can pivot.
I can just convert the above timestamps into UTC using horae (Remember TZ!)

If you are parsing last reboot and hit a crash entry, your logic should follow this flow:

1. Identify the Crash: See crash in the last output for a specific date.
2. Find the Boot Offset: Determine how many boots ago that crash happened (e.g., if it's the entry right below "still running," it's -b -1).
    - Realistically it's always going to be -b -1; This would only be needed if I wanted to support creating the database file for a longer running system at first startup.
        - Easiest fix: Just document that during the first run any crash will not be accounted for.
        - Hard (maybe even hardcore) fix: really calculate the offset. (Should just be adding all reboot events after?)
3. Query the Journal: Run journalctl -b [OFFSET] -n 1.
4. Compare:
    - Start Time: From last reboot.
    - End Time: From the journalctl timestamp.
    - True Uptime: End - Start.
    - True Downtime: Next Boot Start - End.

Example Scenario:

- last reboot says: reboot ... 07:00 - crash (00:45) (Next boot was 07:45).
- journalctl -b -1 -n 1 says: 2026-03-12T07:29:55.
- Result: You now know the power failed at 07:29:55. The system was down for ~15 minutes, not 0 minutes.

If not:

1. Read & parse `last reboot` output.
2. update database file with new information.
3. update output with new uptime (calculated from new data)

JSON example for the `.data` structure:
```json
{
  "metadata": {
    "first_recorded_boot": "2026-01-20T16:03:52Z",
    "last_processed_boot": "2026-03-12T07:00:00Z"
  },
  "statistics": {
    "all_time": { "uptime_percent": 98.42, "total_downtime_seconds": 45120 },
    "current_year": { "year": 2026, "uptime_percent": 99.1 },
    "current_month": { "month": 3, "uptime_percent": 97.5 },
  },
  "history": {
    "2026": {
      "03": {
        "events": [
          {
            "down_at": "2026-03-06T20:59:20Z",
            "up_at": "2026-03-06T21:00:14Z",
            "duration_seconds": 54,
            "type": "clean"
          },
          {
            "down_at": "2026-03-11T02:15:10Z",
            "up_at": "2026-03-11T02:30:00Z",
            "duration_seconds": 890,
            "type": "crash_recovered"
          }
        ],
        "monthly_sum_seconds": 944
      }
    }
  }
}
```

### Key Logic

1. **Granularity:** Statistics are recalculated on every run for the current week, month, and year to provide a health trend indicator alongside the "all-time" record.
2. **Anchor:** `first_recorded_boot` serves as the anchor for calculating the total "possible" uptime.
3. **Crash Recovery:** Sessions marked as `crash_recovered` are identified via `journalctl` probing, ensuring power failures don't skew the data toward 0 downtime.
4. **Calculations:** Uptime % is derived using `horae`: `(Elapsed_Seconds_In_Period - Sum_Of_Downtimes_In_Period) / Elapsed_Seconds_In_Period`.

## First Iteration

Simple loop that waits for the `kill` and `stop` commands.
On startup it logs the time, and when stopped, it takes the logged time to construct a duration of uptime and logs it.
It also creates a file `shutdown_time` with the time of shutdown.
Upon restart, it checks for the file and if it exists, it takes the time from the file to construct a duration of downtime and logs it.

I should be able to construct everything I need form the downtime durations.

This program is supposed to run mainly on the home-lab, maybe the desktop as well.
For easy integration with the system monitoring, I would like one simple main file that records:

- average uptime in %
    * calculated form the first execution of the program - this date and time would need to be saved
- last shutdown

The contents of the main file would be recalculated upon startup of the program.

**Reality Check**

Even if I am producing 100bytes of data per day, it would take around 30 YEARS for the file to reach one gigabyte.
No system, especially no OS, survives for that long.

To delve deeper (and calculate the average uptime) I will need more data. This will be saved in a single persistent file:

- `.data`: A single binary or XFF v3 file containing all historical startup/shutdown events.

Given the "Reality Check" above, a single file is more than sufficient for the lifetime of any modern system, keeping the logic simple and the overhead minimal.

## Handling the "Hard" Shutdown

The current design relies on SIGTERM. One edge case for a home-lab is a power outage or a hard lockup where no signal is sent.
Idea: Have Lasa "touch" a temporary heartbeat file every 5–10 minutes.

## Low pro process

Also I would like to somehow tell the OS that this is a low prio process.

### Research

Read `https://man7.org/linux/man-pages/man7/sched.7.html`.
What I gathered: set `nice value` of the process to its min - don't change the scheduling policy.

The highest `nice value` (exactly what I - tells the scheduler that the process can wait over any other process with a smaller value) is one of three possibilities:

1. 19 - most modern linux systems
2. 20 - some other linux systems
3. 15 - early linux kernels (< 2.0)

#### Determining the highest possible `nice value`
I have found: `https://man7.org/linux/man-pages/man2/getrlimit.2.html`.
So I think I can get the ceiling of the possible values through this sys call.

The returned value is special:

> The actual ceiling for the nice value is calculated as 20 - `rlim_cur`.\
> The useful range for this limit is thus from 1 (corresponding to a nice value of 19) to 40 (corresponding to a nice value of -20).

After an hour its quite simple really:

1. Try `setpriority(20)` first
2. If that fails (probably as most use max 19) do `setpriority(19)`
3. To be funny, have a 15 fallback

BUT:

Maybe do change the policy.
`SCHED_IDLE` only gives CPU time if the cpu would be idle otherwise - exactly what I want really.
Use:
`sched_setscheduler(2)`

again BUT: Shutdown is CPU intensive, so it might not be run before times up for `SIGKILL`

There are other options like:

- `systemd unit file` to run it last
- Use `SCHED_BATCH`

But changing I seem to have been right in not wanting to change the policy but moving towards the nice level. Playing with policies like `BATCH` would make sense if I moved to a multi process design, so a calc process at startup to update the main file (using `BATCH` and `nice 19 / 20), with a small little sleeper for shutdown (using all defaults)

BUT:

```rust
use signal_hook::{consts::SIGTERM, iterator::Signals};
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    // --- PHASE 1: CALCULATIONS ---
    // Set priority to SCHED_BATCH + Nice 19 here if desired
    perform_heavy_calculations();
    println!("Calculations complete.");
    // --- PHASE 2: WAIT ---
    // (Optional) Reset priority to Nice 0 so we're responsive for shutdown
    let mut signals = Signals::new(&[SIGTERM])?;
    // This blocks the main thread completely. 
    // No CPU usage, no threads, just waiting for the signal.
    if let Some(_signal) = signals.forever().next() {
        // --- PHASE 3: LOG ---
        log_shutdown_timestamp();
    }
    Ok(())
}
```
