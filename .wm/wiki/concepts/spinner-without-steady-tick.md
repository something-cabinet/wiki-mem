---
title: 'Failure: indicatif Spinner Without enable_steady_tick Doesn''t Animate'
id: wiki:concepts:spinner-without-steady-tick
type: concept
relates_to:
  - {type: references, target: wiki:howto:indicatif-cli-progress}
---

---
title: Failure: indicatif Spinner Without enable_steady_tick Doesn't Animate
type: concept
id: wiki:concepts:spinner-without-steady-tick
---

# Failure: indicatif Spinner Without enable_steady_tick Doesn't Animate

## What went wrong

Added `indicatif::ProgressBar::new_spinner()` with `set_style()` and `set_message()` but the spinner showed one static frame and never animated. The progress "indicator" was functionally identical to the old bare `println!`.

## Root cause

`ProgressBar::new_spinner()` creates a spinner in the stopped state. Without calling `enable_steady_tick(duration)`, there is no background ticker to advance the spinner animation frame. The `set_message()` writes the first tick string once and never updates it.

The blocking/synchronous call to the work function (e.g., `download_model()`, `rebuild_from_engine()`) prevents any other code from driving the ticker. `enable_steady_tick` spawns a background thread that periodically advances the animation regardless of the main thread's blocking state.

## Fix

Always call `enable_steady_tick` after creating a spinner:

```rust
let spinner = ProgressBar::new_spinner();
spinner.set_style(ProgressStyle::default_spinner()...);
spinner.enable_steady_tick(std::time::Duration::from_millis(100)); // ← required
spinner.set_message("Working...");
// ... blocking work ...
spinner.finish_and_clear();
```

## Prevention

- Treat `enable_steady_tick` as mandatory after every `new_spinner()`
- Test that the spinner animates on a real terminal (PTY), not just in CI
- Review any `ProgressBar::new_spinner()` usage that lacks `enable_steady_tick`

## Time lost

~20 minutes debugging + Oracle review finding.

## See Also

- @wiki/howto/indicatif-cli-progress — full howto with correct pattern
- indicatif docs: https://docs.rs/indicatif