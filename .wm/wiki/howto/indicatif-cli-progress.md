---
title: 'Howto: indicatif Progress Bars for Rust CLI'
id: wiki:howto:indicatif-cli-progress
type: howto
relates_to:
  - {type: references, target: wiki:tasks:review-wm-init--opencodejson-not-generated-during-init}
---

# Howto: indicatif Progress Bars for Rust CLI

## Problem

Long-running CLI operations (model downloads, index rebuilds, file processing) provide no visual feedback. Users stare at a blank terminal wondering if the program hung.

## Solution

Use `indicatif` — a Rust crate for progress bars and spinners in terminal applications.

```toml
# Cargo.toml
indicatif = "0.17"
```

### Spinner (indeterminate progress)

```rust
use indicatif::{ProgressBar, ProgressStyle};

let spinner = ProgressBar::new_spinner();
spinner.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
spinner.enable_steady_tick(std::time::Duration::from_millis(100));
spinner.set_message("Working...");

// ... blocking work ...

spinner.finish_and_clear();
println!("Done!");
```

**CRITICAL:** Always call `enable_steady_tick(duration)` after creating the spinner. Without it, the spinner draws one static frame and never animates — it looks like a frozen `println!`. `enable_steady_tick` starts a background thread that drives animation ticks.

### Progress Bar (determinate progress)

```rust
let pb = ProgressBar::new(total_items);
pb.set_style(
    ProgressStyle::default_bar()
        .template("{bar:40.cyan/blue} {pos}/{len} ({eta})")
        .unwrap()
);
for _ in 0..total_items {
    pb.inc(1);
    // ... process item ...
}
pb.finish_with_message("Complete");
```

## Key Behavior

- Renders to **stderr** by default — safe for piping stdout
- Auto-hides when stderr is not a TTY (piped output)
- Spinners can use custom tick strings for visual variety
- `finish_and_clear()` removes the spinner/progress line
- `finish_with_message()` replaces with a completion message

## Guard Pattern

indicatif handles non-TTY stderr gracefully (auto-hides), but progress bars should still be wrapped behind `--progress` flags or verbose mode for CI environments:

```rust
if !ci_mode {
    let spinner = ProgressBar::new_spinner();
    // ...
}
```

## When to Use

- Model downloads, file processing, index rebuilds
- Any operation that takes >2 seconds
- Multi-step operations where step labels provide useful feedback

## When Not to Use

- Operations that complete in <500ms (the spinner flickers)
- JSON output mode (`--json` flag) — use `finish_and_clear()` before JSON output
- Headless/CI environments without a `--progress` flag

## See Also

- indicatif docs: https://docs.rs/indicatif
- dialoguer for interactive prompts: @wiki/howto/dialoguer-cli-prompts
- WM model download implementation: `apps/wm-cli/src/main.rs`