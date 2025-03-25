# TODOs & Guiding Principals

## Guiding Principals

- Thinking about lib vs app pieces...
  - Library stuff should:
    - use concrete errors
    - use static dispatch/generics
  - App stuff should:
    - use `anyhow` errors
    - use dynamic dispatch/generics
- Thinking about errors...
  - Each command could/should have its own error type. Some behavior across
    those commands will be the same (couldn't find package file, etc), but
    duplicating into one uber-error makes things complicated downstream.

## TODOs

### Phase 1: Base CLI

- [x] Add `clap` CLI app foundation
- [x] Add file system port and adapter

### Phase 2: Configuration Basics

- [x] Add YAML config file loader
- [x] Merge CLI flags with config file for runtime use
- [x] Add command runner port and shell command adapter
- [x] Add running `config validate`

### Phase 3: Package Basics

- [ ] Add YAML package file loader
- [ ] Add running `package validate [name]`
- [ ] Add running `package info [name]`
- [ ] Add running `package list`

### Phase 4: Package Installation

- [ ] Add running `package install`

---

### Later

- [ ] Add running `package create`
- [ ] Add `--dry-run` flag for `package install`
- [ ] Support `use:` to Environments
