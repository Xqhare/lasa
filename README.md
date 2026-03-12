# Lasa

Lasa is a simple rust program to measure and log the system uptime.

It is also the collective name of the ancient Etruscan deities who served as attendants and were often depicted with scrolls, recording events and fate. This project serves as a quiet attendant to the system, recording its lifecycle.

# STOP THE PRESSES

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

JSON just as an example for structure:
```json
{
    "2026": {
        "01": {
            "downtimes": [
                downtime0,
                downtime1,
                downtime2
            ]
            "sum": 69
        }
    }
}

```

"sum" would be total downtime, updated each time a new downtime duration is added to the array.
This enables: Month*days*hours*minutes*seconds - sum
(because of leap days, just use horae again to construct the month duration)
