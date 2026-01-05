# Expert Question 6: Async Script Execution & Channel Management

## The Problem

User scripts run in Bun subprocess, communicate via stdin/stdout JSON:
1. App spawns `bun run --preload sdk.ts script.ts`
2. Script sends prompts to stdout (JSON lines)
3. App parses, displays prompt, waits for user input
4. App sends response to script's stdin
5. Script continues, may send more prompts
6. Script exits, app closes prompt

## Specific Concerns

1. **Bounded Channel (10 messages)**: We use `sync_channel(10)` for responses. If script sends prompts faster than user responds, it blocks. Intentional backpressure or bug?

2. **Stderr Tee**: We forward stderr to logging AND buffer for end-of-script capture. Race condition: process can exit while we're still reading stderr.

3. **Process Cleanup**: We track PID and send SIGTERM on cancel. But `bun` may spawn child processes. Do we need process group handling?

4. **No Timeout**: Scripts can hang forever. Should we add a configurable timeout? What's the UX for timeout (kill silently vs. prompt user)?

5. **Selected Text Injection**: Before script runs, we capture system selection and inject into global scope. This has race conditions on fast script switching.

## Questions for Expert

1. Is bounded channel the right pattern here? What capacity?
2. How do we guarantee stderr is fully captured before reporting script exit?
3. Should we use process groups (`setpgid`) to ensure clean child process cleanup?
4. What's the recommended pattern for subprocess timeout in Rust?
5. Should script execution be cancellable mid-prompt, or only between prompts?

