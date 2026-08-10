use std::collections::BTreeMap;

use compact_str::{CompactString, CompactStringExt as _, format_compact};
use indoc::formatdoc;
use itertools::Itertools as _;
use mitsein::btree_set1::BTreeSet1;
use mulan_config::Language;

pub fn generate(data: &mulan_parser::Output, config: &mulan_config::Config) -> String {
    todo!();
}

#[derive(Debug)]
pub struct Bindings<'src> {
    locales: &'src [Language],
    t: Module<'src>,
}

impl Bindings<'_> {
    fn generate(&self) -> String {
        formatdoc! {"
            pub enum Locale {{
                {locale_variants}
            }}

            {mod_t}
            ",
            locale_variants = indent::indent_by(4, {
                self.locales
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
        match &self.fields {
            Some(fields) => formatdoc! {"
                impl {name} {{
                    pub fn get_in(&self, locale: Locale) -> String {{
                        match locale {{
                            _ => todo!(),
                        }}
                    }}
                }}\
                ",
            },
            None => formatdoc! {"
                impl {name} {{
                    pub fn get_in(&self, locale: Locale) -> &'static str {{
                        match locale {{
                            _ => todo!(),
                        }}
                    }}
                }}\
                ",
            },
        }
    }
}
