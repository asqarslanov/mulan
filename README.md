# Mulan

A multi-language i18n framework.

## Get Started

Use
[`cargo-install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
to install the `mulan` CLI to your
[Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html)&rsquo;s bin
directory (most likely, `~/.cargo/bin/`).

```sh
cargo install --git=https://github.com/asqarslanov/mulan.git
```

> You are also encouraged to add `$HOME/.cargo/bin` to your `$PATH`.

Then, `cd` into your project and type this command to start using Mulan:

```sh
mulan init
```

## Showcase

- `locales/en.yaml`

  ```yaml
  app-name: "Foo"
  ui:
    greeting: "Hello, {name}!"
  general:
    author: "Made by {t.app-name} Inc."
  ```

- `locales/fr.yaml`

  ```yaml
  app-name: "Toto"
  ui:
    greeting: "Bonjour, {name}!"
  general:
    author: "Fabriqué par {t.app-name} SA"
  ```
