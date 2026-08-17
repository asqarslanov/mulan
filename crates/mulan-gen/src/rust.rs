//! # Mulan Gen / Rust
//!
//! Generates bindings for the Rust programming language.
//!
//! See [`generate`].

use std::collections::{BTreeMap, BTreeSet};
use std::iter;

use compact_str::{CompactString, CompactStringExt as _, ToCompactString as _, format_compact};
use indoc::formatdoc;
use itertools::Itertools as _;
use mitsein::btree_set1::BTreeSet1;

use crate::AUTO_GENERATED_COMMENT;

const INDENT: usize = 4;
const VARIABLE_CASE: mulan_config::Case = mulan_config::Case::Snake;
const MODULE_CASE: mulan_config::Case = mulan_config::Case::Snake;
const TYPE_CASE: mulan_config::Case = mulan_config::Case::Pascal;

/// Returns a Rust source code string that can be used in a standalone file.
#[must_use]
pub fn generate(config: &mulan_config::Config, data: &mulan_parser::Output) -> String {
    Bindings {
        t: Module::new(&data.root, None),
    }
    .generate(config)
}

#[derive(Debug)]
struct Bindings<'src> {
    t: Module<'src>,
}

impl Bindings<'_> {
    fn generate(&self, config: &mulan_config::Config) -> String {
        formatdoc! {"
            // {auto_generated_comment}

            //! # Mulan

            #![allow(warnings)]

            #[rustfmt::skip]

            {enum_locale}

            {mod_t}
            ",
            auto_generated_comment = indent::indent_with("// ", AUTO_GENERATED_COMMENT),
            enum_locale = Self::enum_locale(config),
            mod_t = self.t.generate(config, None),
        }
    }

    fn enum_locale(config: &mulan_config::Config) -> String {
        let doc_comment = indent::indent_all_with(
            "/// - ",
            formatdoc! {"
                [`{main}`](Self::{main}) (main)
                {other}\
                ",
                main = config.main_locale.tag_pascal_case(),
                other = {
                    config
                        .locales_except_main()
                        .map(|lang| format_compact!(
                            "[`{variant}`](Self::{variant})",
                            variant = lang.tag_pascal_case(),
                        ))
                        .join_compact("\n")
                },
            },
        );
        formatdoc! {"
            {doc_comment}
            #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum Locale {{
                {variants}
            }}\
            ",
            variants = indent::indent_by(INDENT, {
                config
                    .locales
                    .iter()
                    .map(|lang| formatdoc! {"
                        /// {tag} / {name}\
                        {default_attr}
                        {variant},\
                        ",
                        tag = lang.tag(),
                        name = lang.name(),
                        default_attr = if lang == &config.main_locale {
                            "\n#[default]"
                        } else {
                            ""
                        },
                        variant = lang.tag_pascal_case(),
                    })
                    .join_compact("\n\n")
            }),
        }
    }
}

#[derive(Debug)]
struct Module<'src> {
    structs: BTreeMap<mulan_parser::Key, Struct<'src>>,
    submodules: BTreeMap<mulan_parser::Key, Self>,
}

impl<'src> Module<'src> {
    fn new(
        namespace: &'src mulan_parser::Namespace,
        parent_key: Option<&mulan_parser::Key>,
    ) -> Self {
        let mut structs = BTreeMap::new();
        let mut submodules = BTreeMap::new();
        for (key, node) in namespace.iter(parent_key) {
            use mulan_parser::Node as N;
            match node {
                N::Message(msg) => {
                    structs.insert(key, Struct::new(msg));
                }
                N::Namespace(ns) => {
                    let submodule = Module::new(ns, Some(&key));
                    submodules.insert(key, submodule);
                }
            }
        }
        Self {
            structs,
            submodules,
        }
    }

    fn generate(&self, config: &mulan_config::Config, key: Option<&mulan_parser::Key>) -> String {
        let mut module_contents = Vec::new();
        if !self.structs.is_empty() {
            module_contents.push(self.gen_structs(config));
        }
        if !self.submodules.is_empty() {
            module_contents.push(self.gen_submodules(config));
        }
        #[expect(clippy::option_if_let_else, reason = "borrowed values")]
        let (doc_comment, name): (&str, &str) = if let Some(key) = key {
            (
                &format_compact!("`{name}`", name = key.to_compact_string1(config.key_case)),
                &key.name().to_compact_string1(MODULE_CASE),
            )
        } else {
            ("The root namespace.", "t")
        };
        if module_contents.is_empty() {
            formatdoc! {"
                /// {doc_comment}
                pub mod {name} {{}}\
                ",
                doc_comment = indent::indent_with("/// ", doc_comment),
            }
        } else {
            formatdoc! {"
                /// {doc_comment}
                pub mod {name} {{
                    use super::Locale;

                    {items}
                }}\
                ",
                doc_comment = indent::indent_with("/// ", doc_comment),
                items = indent::indent_by(INDENT, module_contents.join("\n\n")),
            }
        }
    }

    fn gen_structs(&self, config: &mulan_config::Config) -> String {
        self.structs
            .iter()
            .map(|(name, structure)| structure.generate(config, name))
            .join("\n\n")
    }

    fn gen_submodules(&self, config: &mulan_config::Config) -> String {
        self.submodules
            .iter()
            .map(|(name, submodule)| submodule.generate(config, Some(name)))
            .join("\n\n")
    }
}

#[derive(Debug)]
struct Struct<'src> {
    translations: &'src mulan_parser::Translations,
    fields: Option<BTreeSet1<&'src mulan_parser::Parameter>>,
}

impl<'src> Struct<'src> {
    fn new(translations: &'src mulan_parser::Translations) -> Self {
        Self {
            translations,
            fields: translations.parameter_set(),
        }
    }

    fn generate(&self, config: &mulan_config::Config, key: &mulan_parser::Key) -> String {
        let name = &key.name().to_compact_string1(TYPE_CASE);
        formatdoc! {"
            {doc_comment}
            pub struct {name}{lifetimes}{block}

            {impl_block}\
            ",
            doc_comment = self.doc_comment(config, key),
            lifetimes = self.gen_lifetimes(),
            block = self.gen_block(),
            impl_block = self.gen_impl(name),
        }
    }

    fn doc_comment(&self, config: &mulan_config::Config, key: &mulan_parser::Key) -> String {
        let preview = self.translations.markdown_preview();
        let preview = preview.as_ref().map_or("_empty message_", AsRef::as_ref);
        formatdoc! {"
            /// `{key}`
            ///
            /// {markdown_preview}\
            ",
            key = key.to_compact_string1(config.key_case),
            markdown_preview = indent::indent_with("/// ", preview),
        }
    }

    fn gen_lifetimes(&self) -> CompactString {
        let Some(fields) = &self.fields else {
            return CompactString::default();
        };
        format_compact!(
            "<{}>",
            fields
                .iter1()
                .map(|subkey| format_compact!(
                    "'{name}",
                    name = subkey.to_compact_string1(VARIABLE_CASE),
                ))
                .join_compact(", "),
        )
    }

    fn gen_lifetime_placeholders(&self) -> CompactString {
        let Some(fields) = &self.fields else {
            return CompactString::default();
        };
        format_compact!(
            "<{}>",
            iter::repeat_n("'_", fields.len().get()).join_compact(", "),
        )
    }

    fn gen_block(&self) -> String {
        let Some(fields) = &self.fields else {
            return ";".to_owned();
        };
        formatdoc! {"
             {{
                {fields}
            }}\
            ",
            fields = indent::indent_by(INDENT, {
                fields
                    .iter1()
                    .map(|subkey| format_compact!(
                        "pub {name}: &'{name} str,",
                        name = subkey.to_compact_string1(VARIABLE_CASE),
                    ))
                    .into_iter()
                    .join("\n")
            }),
        }
    }

    fn gen_impl(&self, name: &str) -> String {
        let matching = |allow_str: bool| -> String {
            self.translations
                .others
                .iter()
                .map(move |(lang, msg)| {
                    format!(
                        "Locale::{locale} => {result},",
                        locale = lang.tag_pascal_case(),
                        result = generate_message(msg, allow_str),
                    )
                })
                .chain(iter::once(format!(
                    "_ => {result},",
                    result = generate_message(&self.translations.main, allow_str),
                )))
                .join("\n")
        };
        let lifetime_placeholders = self.gen_lifetime_placeholders();
        match &self.fields {
            Some(_) => formatdoc! {"
                impl {name}{lifetime_placeholders} {{
                    pub fn get_in(&self, locale: Locale) -> String {{
                        match locale {{
                            {}
                        }}
                    }}
                }}\
                ",
                indent::indent_by(3 * INDENT, matching(false)),
            },
            None => formatdoc! {"
                impl {name} {{
                    pub fn get_in(&self, locale: Locale) -> &'static str {{
                        match locale {{
                            {}
                        }}
                    }}
                }}\
                ",
                indent::indent_by(3 * INDENT, matching(true)),
            },
        }
    }
}

fn generate_message(template: &mulan_parser::Template, allow_str: bool) -> CompactString {
    if let Some(text) = template.try_as_plain_text() {
        return format_compact!(
            "\"{contents}\"{tail}",
            contents = text.escape_debug(),
            tail = if allow_str { "" } else { ".to_owned()" },
        );
    }
    let contents = {
        use mulan_parser::{Tag as T, TemplatePart as P};
        template
            .iter()
            .map(|part| match part {
                P::Text(text) => text.escape_debug().to_compact_string(),
                P::Tag(T::Parameter(parameter)) => {
                    format_compact!(
                        "{{{name}}}",
                        name = parameter.to_compact_string1(VARIABLE_CASE),
                    )
                }
            })
            .collect::<CompactString>()
    };
    let named_parameters = {
        template
            .parameter_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|param| {
                format_compact!(
                    "{name} = self.{name}",
                    name = param.to_compact_string1(VARIABLE_CASE),
                )
            })
            .join(", ")
    };
    format_compact!("format!(\"{contents}\", {named_parameters})")
}
