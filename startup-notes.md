# Lasa

Lasa is the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

Create a simple program to measure and log the system uptime.

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
