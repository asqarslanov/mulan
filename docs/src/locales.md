# Locales

```json5
{
  plain: "Simple text",
  accept: "Accept {baz} :rocket:",
  ref: "Idk, {&plain}",
  date: "It's {d:date}",
  escape: "Just a text with {{braces}}",

  ext_1: {
    // parameters
    params: ["foo", { name: "bar", type: "int" }, "baz: date"],
    // text
    txt: "{baz} {foo} {bar}",
    // options
    opts: { prettify: false, emoji: false, params_as_struct: true },
  },

  ext_2: {
    params: ["name", "apples", "pens"],
    plural: ["apples", "pens"],
    txt: {
      one: {
        one: "",
        other: "",
      },
      other: {
        one: "",
        other: "",
      },
    },
  },

  extract_1: "{#foo}One{/foo} {#bar}two{/bar}",
  extract_2: "{# foo}One{/} {#}two{/}",

  import_0: true,
  import_1: ["import", true],
  import_2: ["import", "path/to/file"],
  import_3: ["import", "path/to/file.json5"],
  import_4: ["import", { src: "path/to/file" }],
  import_5: ["import", { src: "path/to/file.json5" }],

  secion: "",
}
```
