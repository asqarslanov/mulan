use std::collections::BTreeMap;
use std::iter;

use compact_str::{CompactString, CompactStringExt as _, ToCompactString as _, format_compact};
use indoc::formatdoc;
use itertools::Itertools as _;
use mitsein::btree_set1::BTreeSet1;

pub fn generate(data: &mulan_parser::Output, config: &mulan_config::Config) -> String {
    todo!();
}

#[derive(Debug)]
pub struct Bindings<'src> {
    t: Module<'src>,
}

impl Bindings<'_> {
    fn generate(&self, config: &mulan_config::Config) -> String {
        formatdoc! {"
            pub enum Locale {{
                {locale_variants}
            }}

            {mod_t}
            ",
            locale_variants = indent::indent_by(4, {
                config
                    .locales
                    .iter()
                    .map(|lang| format_compact!("{},", lang.tag_pascal_case()))
                    .join_compact("\n")
            }),
            mod_t = self.t.generate("t"),
        }
    }
}

#[derive(Debug)]
struct Module<'src> {
    structs: BTreeMap<&'src mulan_parser::Subkey, Struct<'src>>,
    submodules: BTreeMap<&'src mulan_parser::Subkey, Module<'src>>,
}

impl Module<'_> {
    fn generate(&self, name: &str) -> String {
        formatdoc! {"
            pub mod {name} {{
                use crate::Locale;

                {structs}

                {submodules}
            }}\
            ",
            structs = indent::indent_by(4, self.gen_structs()),
            submodules = indent::indent_by(4, self.gen_submodules()),
        }
    }

    fn gen_structs(&self) -> String {
        self.structs
            .iter()
            .map(|(name, structure)| structure.generate(&name.to_pascal_case()))
            .join("\n\n")
    }

    fn gen_submodules(&self) -> String {
        self.submodules
            .iter()
            .map(|(name, submodule)| submodule.generate(&name.to_snake_case()))
            .join("\n\n")
    }
}

#[derive(Debug)]
struct Struct<'src> {
    translations: mulan_parser::Translations,
    fields: Option<BTreeSet1<&'src mulan_parser::Subkey>>,
}

impl Struct<'_> {
    fn generate(&self, name: &str) -> String {
        formatdoc! {"
            pub struct {name}{lifetimes}{block}

            {impl_block}\
            ",
            lifetimes = self.gen_lifetimes(),
            block = self.gen_block(),
            impl_block = self.gen_impl(name),
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
                .map(|subkey| format_compact!("'{name}", name = subkey.to_kebab_case()))
                .join_compact(" "),
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
            fields = indent::indent_by(4, {
                fields
                    .iter1()
                    .map(|subkey| format_compact!(
                        "pub {name}: &'{name} str,",
                        name = subkey.to_kebab_case(),
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
                .collect()
        };
        match &self.fields {
            Some(_) => formatdoc! {"
                impl {name} {{
                    pub fn get_in(&self, locale: Locale) -> String {{
                        match locale {{
                            {}
                        }}
                    }}
                }}\
                ",
                indent::indent_by(12, matching(false)),
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
                indent::indent_by(12, matching(true)),
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
        use mulan_parser::TemplatePart as P;
        template
            .iter()
            .map(|part| match part {
                P::Text(text) => text.escape_debug().to_compact_string(),
                P::Placeholder(parameter) => {
                    format_compact!("{{{name}}}", name = parameter.to_snake_case())
                }
            })
            .collect::<CompactString>()
    };
    let named_parameters = {
        template
            .parameters()
            .map(|param| format_compact!("{name} = self.{name}", name = param.to_snake_case()))
            .join(", ")
    };
    format_compact!("format!(\"{contents}\", {named_parameters})")
}
