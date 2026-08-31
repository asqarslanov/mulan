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

> For now, the only supported compilation target is [Rust](https://rust-lang.org/).

Suppose you have such locale files.

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

You can generate i18n bindings with this command.

```sh
mulan gen
```

Then, you can import and use the generated bindings in your code.

```rs
use mulan::{Locale, t};

let name_en: &'static str = t::AppName.get_in(Locale::default());
println!("{name_en}");

for locale in [Locale::En, Locale::Fr] {
    let greeting: String = t::ui::Greeting { name: "Mushu" }.get_in(locale);
    println!("{greeting}");
}

let author_fr: &'static str = t::general::Author.get_in(Locale::Fr);
println!("{author_fr}");
```

- ```stdout
  Foo
  Hello, Mushu!
  Bonjour, Mushu!
  Fabriqué par Toto SA
  ```
