---
title: "Design Pattern Alignment — Naming, Structure, Conventions"
page_type: spec
status: draft
tags: [spec, refactor, architecture, naming, patterns]
relates_to:
  - {type: answers, target: wiki:decisions/design-pattern-alignment-file-name-role}
  - {type: answers, target: wiki:decisions/design-pattern-alignment-barrel-files}
  - {type: answers, target: wiki:decisions/design-pattern-alignment-model-service-split}
  - {type: answers, target: wiki:decisions/design-pattern-alignment-constants}
  - {type: implements, target: wiki:tasks/design-pattern-alignment-fr-1-rename-files}
  - {type: implements, target: wiki:tasks/design-pattern-alignment-fr-2-barrel-files}
  - {type: implements, target: wiki:tasks/design-pattern-alignment-fr-3-split-mixed}
  - {type: implements, target: wiki:tasks/design-pattern-alignment-fr-4-extract-constants}
---

## Overview

Align the codebase with design pattern conventions from gehenna-app: every file's name encodes its pattern role (Builder, Factory, Service, Repository, Model, etc.). Aggressively split models from services from helpers. Use Barrel files (`mod.rs`) to present clean public APIs.

Reference: @wiki/reference/design-patterns

## Locked Decisions

Each locked decision below has (or will have) a corresponding Decision page at `wiki:decisions/design-pattern-alignment-*` with full ADR context.

- **D1 — File Name = Pattern Role**: Every Rust file under `src/` MUST end with its pattern role suffix (`Model`, `Service`, `Helper`, `Constant`, `Repository`, `Builder`, `Factory`, `Proxy`, `Mediator`). A file named `update.rs` tells you nothing. `PageUpdateBuilderService.rs` tells you it's a Builder-pattern Service for Page updates. This convention comes from gehenna-app's `CONVENTIONS.md` where every module filename encodes its architectural role.
- **D2 — Barrel Files Required**: Every module directory MUST have a `mod.rs` that re-exports all public items. No consumer imports from individual files within a directory — always through the Barrel. This eliminates fragile import paths and makes refactoring safe (move a file → update Barrel, no consumer changes).
- **D3 — Models vs Services Split**: A struct and its methods are separate concerns — the struct definition (`XxxModel.rs`) and the operations on it (`XxxService.rs`). If you can't name the service without mentioning the model, they're the same concern — keep them. But if a type has 5+ associated functions, extract them to a Service file.
- **D4 — Constants in Dedicated Files**: Static data (`const`, `OnceLock`, `LazyLock`, `RustEmbed`) goes in `*Constant.rs`. A model file should not contain embedded assets or lazily-initialized regexes.

## Naming Convention

Pattern: `{Domain}{Role}{Pattern}` where:
- `Domain` = the business domain (Page, Task, Source, Version, Search, Graph, Memory)
- `Role` = Model / Service / Helper / Constant / Repository / Builder / Factory
- `Pattern` = Builder / Factory / Strategy / State / Command / Observer / Proxy / Adapter / etc.

### Examples

| Current | Should be | Reason |
|---|---|---|
| `page/update.rs` | `page/PageUpdateBuilderService.rs` | Builder pattern, acts as service |
| `page/crud.rs` | `page/PageCrudService.rs` | Service handling CRUD |
| `page/yaml.rs` | `page/YamlHelper.rs` | Utility helpers |
| `page/migration.rs` | `page/MigrationService.rs` | One-shot migration service |
| `page/recovery.rs` | `page/TimerRecoveryService.rs` | Recovery service |
| `page/path.rs` | `page/PagePathHelper.rs` | Path resolution helpers |
| `version/store.rs` | `version/VersionStoreRepository.rs` | Repository pattern |
| `version/field_change.rs` | `version/FieldChangeModel.rs` | Pure model |
| `version/task_version.rs` | `version/TaskVersionModel.rs` | Pure model |
| `version/doc_version.rs` | `version/DocVersionModel.rs` | Pure model |
| `version/task_history.rs` | `version/TaskVersionHistoryModel.rs` | Pure model |
| `version/doc_history.rs` | `version/DocVersionHistoryModel.rs` | Pure model |
| `version/mod.rs` | `version/mod.rs` (Barrel) | `compute_field_changes` → helper |
| `skill/skill.rs` | `skill/SkillModel.rs` | Pure model |
| `skill/engine.rs` | `skill/SkillEngineService.rs` | Service with engine lifecycle |
| `skill/trigger_event.rs` | `skill/TriggerEventModel.rs` | Model with FromStr |
| `skill/trigger_config.rs` | `skill/TriggerConfigModel.rs` | Model |
| `skill/tool_spec.rs` | `skill/SkillToolSpecModel.rs` | Model |
| `skill/assets.rs` | `skill/SkillAssetsConstant.rs` | Constants (RustEmbed) |
| `skill/frontmatter.rs` | `skill/SkillFrontmatterParserHelper.rs` | Parser helper |
| `engine/page_type.rs` | `engine/PageTypeModel.rs` | Pure enum model |
| `engine/edge_type.rs` | `engine/EdgeTypeModel.rs` | Pure enum model |
| `engine/time_entry.rs` | `engine/TimeEntryModel.rs` | Pure model |
| `engine/audit_event.rs` | `engine/AuditEventModel.rs` | Pure model |
| `engine/relation.rs` | `engine/RelationHelper.rs` | Serde helpers + parse fn |
| `memory/layer.rs` | `engine/memory/MemoryLayerModel.rs` | Enum model |
| `memory/entry.rs` | `engine/memory/MemoryEntryModel.rs` | Model |
| `source/state.rs` | `engine/source/SourceStateModel.rs` | Enum model |
| `source/entry.rs` | `engine/source/SourceEntryModel.rs` | Model |
| `template/prompt.rs` | `engine/template/TemplatePromptModel.rs` | Model |
| `template/action.rs` | `engine/template/TemplateActionModel.rs` | Model |
| `template/config.rs` | `engine/template/TemplateConfigModel.rs` | Model |
| `page_data/` | All → `engine/page_data/*Model.rs` | All are data types |
| `engine/state.rs` | `engine/EngineStateMediator.rs` | Mediator pattern |
| `engine/scheduler.rs` | `engine/IndexSchedulerService.rs` | Service |
| `engine/write_channel.rs` | `engine/WriteChannelProxy.rs` | Proxy over file I/O |

## Barrel Files

Every module directory MUST have a `mod.rs` (Barrel file) that re-exports its public API. No client code should import from individual files within a directory — always import from the module root.

### Good
```rust
// In some_consumer.rs
use crate::page::{PageCrudService, PageUpdateBuilderService, PagePathHelper};

// In page/mod.rs
pub use page_crud_service::*;
pub use page_update_builder_service::*;
pub use page_path_helper::*;
```

### Bad
```rust
// In some_consumer.rs  
use crate::page::page_crud_service::create_page;
```

## Module Structure

Every module should have at most 4 sub-directories/files:

```
domain/
  mod.rs               ← Barrel: re-exports everything public
  DomainModel.rs       ← Data types: structs, enums, their impls
  DomainService.rs     ← Business logic: functions operating on models
  DomainHelper.rs      ← Pure utility functions (no domain deps)
  DomainConstant.rs    ← Constants, static configs, embedded assets
```

Larger domains (with multiple models or services) can be further split by pattern:

```
page/
  mod.rs
  PageModel.rs
  PageCrudService.rs
  PageUpdateBuilderService.rs  
  PageYamlHelper.rs
  PagePathHelper.rs
  MigrationService.rs
  TimerRecoveryService.rs
```

## Rules

### R1: File Name = Pattern Role
Every file name MUST be snake_case and end with its role suffix:
- `*_model.rs` — pure data (structs, enums, their impls). No external deps beyond serde.
- `*_service.rs` — business logic functions. Depends on models + repositories.
- `*_helper.rs` — stateless utility functions. No domain imports.
- `*_constant.rs` — constants, static configs, embedded assets (RustEmbed, LazyLock).
- `*_repository.rs` — data access (reads/writes storage). Depends on models.
- `*_builder_service.rs` — constructs complex objects step by step.
- `*_factory_service.rs` — creates objects with construction logic.
- `*_proxy.rs` — controls access to another object (lazy init, caching, permissions).
- `*_mediator.rs` — coordinates multiple objects.
- `*_strategy.rs` — family of interchangeable algorithms.
- `*_state_machine.rs` — state machine with transitions.

### R2: Barrel Everything
Every `mod.rs` in a module directory:
1. Declares sub-modules as `mod model_name;`
2. Re-exports everything: `pub use model_name::*;`

### R3: One Pattern Per File
Each file contains exactly one primary pattern implementation. If a struct is a model AND has service methods, split them — model in `XxxModel.rs`, methods in `XxxService.rs`.

### R4: Constants Separated
Any `const`, `static`, `LazyLock`, `OnceLock`, `RustEmbed` goes in a `*Constant.rs` file, not in a model or service file.

### R5: Naming Within Files
- Public types: `PageModel`, `PageCrudService`, `PageYamlHelper`
- File name matches primary type name (minus suffix role)
- Enums used as models: `EdgeTypeModel`, `PageTypeModel`, `SourceStateModel`

## Fix Plan

### Phase 1: Rename files to pattern convention
Rename all files in `engine/`, `page/`, `version/`, `skill/`, `parser/`, `source/`, `reference/`, `task/`, `graph/` to match naming convention. No logic changes.

### Phase 2: Barrel file audit
Ensure every directory has a `mod.rs` that re-exports everything. Remove direct path imports.

### Phase 3: Split mixed files
Files that mix patterns (e.g., a file with both Model and Service) get split:
- `engine/state.rs` → `EngineStateMediator.rs` (mediator) + split indexes/search/config into smaller bundles
- `mcp/tools/task/` → action enum (Command), handlers (Service) separated
- `version/mod.rs` → `compute_field_changes` moved to `FieldChangeHelper.rs`
- `skill/mod.rs` → functions moved to `SkillParserHelper.rs`

### Phase 4: Constants extraction
Move all `const`, `static`, `OnceLock`, `LazyLock` to `*Constant.rs` files.

## Acceptance Criteria

- [ ] AC-1: Every file under `src/` ends with a role suffix (Model/Service/Helper/Constant/Repository/Builder/Factory/Proxy/Mediator)
- [ ] AC-2: Every module directory has a Barrel `mod.rs` re-exporting all public items
- [ ] AC-3: No direct file-path imports exist (all go through `mod.rs`)
- [ ] AC-4: No file mixes Model + Service (split if found)
- [ ] AC-5: All constants moved to `*Constant.rs` files
- [ ] AC-6: `cargo build --all-features` succeeds
- [ ] AC-7: `cargo test` passes same count
- [ ] AC-8: `cargo clippy` no new warnings

## Non-Goals

- No logic changes in Phase 1-2 (pure renames + re-exports)
- No behavioral changes to any type or function
- Barrel files may re-export under shorter names where ergonomic
