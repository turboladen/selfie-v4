# selfie

Selfie is a (meta) package manager that orchestrates package installations
across different package managers and environments. It allows users to define
their own package installation rules per environment, managing dependencies and
providing clear feedback about installation progress.

## Packages

A Selfie package contains information about how you want to install that package
on each of your environments. It lets you define some metadata about the
package, plus commands for how to, in each given environment, a) `check` if the
package is already installed, and b) how to install it. Selfie's CLI then
provides commands to execute those commands in your environment.

For example, if you have a package file for installing `ripgrep` that looks
like:

```yaml
# packages/ripgrep.yaml
---
name: ripgrep
version: 0.1.0
homepage: https://github.com/BurntSushi/ripgrep
description: ripgrep recursively searches directories for a regex pattern while respecting your gitignore

environments:
  macos:
    install: brew install ripgrep
    check: which rg
    dependencies:
      - homebrew
  arch-linux:
    install: sudo pacman -S ripgrep
    check: which rg
```

...you've got yourself a Selfie package named `ripgrep` that you can install in
any of two environments: `macos` and `arch-linux`.

When you run `selfie package install ripgrep`, and your Selfie config file says
your current environment is `macos`, then Selfie will:

1. execute the `environments.macos.check` command to see if `ripgrep` is already
   installed. If that returns `true`, Selfie's job is done, and it exits; if
   not, it...
2. does this same `check` and `install` process for each of the `dependencies`
   its dependency tree, then...
3. Executes the `environments.macos.install` command to install the package.

As a note, you can also run `selfie package check ripgrep` to simply execute
`environments.macos.check`.

