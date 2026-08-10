---
title: 'Howto: dialoguer for Rust CLI Interactive Prompts'
id: wiki:howto:dialoguer-cli-prompts
type: howto
relates_to:
  - {type: references, target: wiki:tasks:review-wm-init--opencodejson-not-generated-during-init}
---

# Howto: dialoguer for Rust CLI Interactive Prompts

## Problem

Rust CLI tools often need interactive user input (confirmations, selections, multi-select). Bare `println!` + `std::io::stdin().read_line()` produces unstyled, hard-to-use prompts with no arrow-key navigation, no validation, and no multi-select support.

## Solution

Use `dialoguer` — a Rust crate providing styled interactive prompts with terminal-aware rendering.

```toml
# Cargo.toml
dialoguer = "0.11"
```

### Confirm (yes/no)

```rust
use dialoguer::{theme::ColorfulTheme, Confirm};

let theme = ColorfulTheme::default();
let enabled = Confirm::with_theme(&theme)
    .with_prompt("Enable feature X?")
    .default(false)
    .interact()
    .unwrap_or(false);
```

### Select (single choice from list)

```rust
use dialoguer::{theme::ColorfulTheme, Select};

let theme = ColorfulTheme::default();
let options = ["Option A", "Option B", "Option C"];
let choice = Select::with_theme(&theme)
    .with_prompt("Select an option")
    .items(&options)
    .default(0)
    .interact()
    .unwrap_or(0);
```

### MultiSelect (multiple choices with space to toggle)

```rust
use dialoguer::{theme::ColorfulTheme, MultiSelect};

let theme = ColorfulTheme::default();
let items = ["Item 1", "Item 2", "Item 3"];
let selections = MultiSelect::with_theme(&theme)
    .with_prompt("Select items (Space to toggle, Enter to confirm)")
    .items(&items)
    .interact()
    .unwrap_or_default();
// selections is Vec<usize> of selected indices
```

### Input (free text)

```rust
use dialoguer::{theme::ColorfulTheme, Input};

let name: String = Input::with_theme(&theme)
    .with_prompt("Enter your name")
    .interact_text()
    .unwrap_or_default();
```

## Key Behavior

- All prompts are styled with `ColorfulTheme` by default (arrows, colors, clear layout)
- Renders to **stderr** — safe for piping stdout to files or JSON consumers
- Arrow keys navigate, Enter confirms, Space toggles (MultiSelect)
- `unwrap_or(default)` provides sensible fallbacks when stdin is piped or non-TTY
- Honors `NO_COLOR` and `CLICOLOR` environment variables
- `is_terminal::is_terminal(stdin)` guard should wrap interactive sections

## Guard Pattern

Always guard interactive prompts with a terminal check and `--no-wizard` / `--yes` flags:

```rust
if !no_wizard && is_terminal::is_terminal(std::io::stdin()) {
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("...")
        .interact()
        .unwrap_or(false);
}
```

## When to Use

- CLI setup wizards (`wm init`, package init, project scaffold)
- Confirmation prompts before destructive operations
- Multi-select for platform/feature selection

## When Not to Use

- Non-interactive/headless environments (use flags or env vars)
- When stdin is piped (always check `is_terminal`)
- Simple yes/no that must be scriptable (use `--force` / `--yes` flags instead)

## See Also

- dialoguer docs: https://docs.rs/dialoguer
- indicatif for progress bars: @wiki/howto/indicatif-cli-progress
- WM init wizard implementation: `apps/wm-cli/src/main.rs`