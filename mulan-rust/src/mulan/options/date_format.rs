use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use private::{AddComponent, DateFormatBuilder, Unit};

pub struct DateFormat {
    units: Box<[Unit]>,
}

impl DateFormat {
    #[must_use]
    pub fn builder() -> DateFormatBuilder<AddComponent> {
        const UNITS: usize = 5;
        DateFormatBuilder {
            units: Vec::with_capacity(UNITS),
            state: PhantomData,
        }
    }
}

impl Debug for DateFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DateFormat")
            .field(
                &self
                    .units
                    .iter()
                    .map(<&'static str>::from)
                    .collect::<Box<str>>(),
            )
            .finish()
    }
}

mod private {
    use std::marker::PhantomData;

    use strum::IntoStaticStr;

    use super::DateFormat;

    pub struct DateFormatBuilder<S: State> {
        pub units: Vec<Unit>,
        pub state: PhantomData<S>,
    }

    pub trait State {}

    impl State for AddComponent {}
    pub enum AddComponent {}

    impl State for AddSeparator {}
    pub enum AddSeparator {}

    #[derive(IntoStaticStr)]
    #[strum(serialize_all = "lowercase")]
    pub enum Component {
        Yy,
        Yyyy,
        M,
        Mm,
        Mmm,
        Mmmm,
        D,
        Dd,
        Ddd,
        Dddd,
    }

    #[derive(IntoStaticStr)]
    pub enum Separator {
        #[strum(to_string = "/")]
        Slash,
        #[strum(to_string = ".")]
        Dot,
        #[strum(to_string = "-")]
        Dash,
        #[strum(to_string = " ")]
        Space,
    }

    pub enum Unit {
        Component(Component),
        Separator(Separator),
    }

    impl From<&Unit> for &'static str {
        fn from(value: &Unit) -> Self {
            match value {
                Unit::Component(it) => it.into(),
                Unit::Separator(it) => it.into(),
            }
        }
    }

    impl DateFormatBuilder<AddComponent> {
        fn add_component(mut self, component: Component) -> DateFormatBuilder<AddSeparator> {
            self.units.push(Unit::Component(component));
            DateFormatBuilder {
                units: self.units,
                state: PhantomData,
            }
        }

        pub fn yy(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Yy)
        }

        pub fn yyyy(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Yyyy)
        }

        pub fn m(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::M)
        }

        pub fn mm(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Mm)
        }

        pub fn mmm(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Mmm)
        }

        pub fn mmmm(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Mmmm)
        }

        pub fn d(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::D)
        }

        pub fn dd(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Dd)
        }

        pub fn ddd(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Ddd)
        }

        pub fn dddd(self) -> DateFormatBuilder<AddSeparator> {
            self.add_component(Component::Dddd)
        }
    }

    impl DateFormatBuilder<AddSeparator> {
        fn add_separator(mut self, separator: Separator) -> DateFormatBuilder<AddComponent> {
            self.units.push(Unit::Separator(separator));
            DateFormatBuilder {
                units: self.units,
                state: PhantomData,
            }
        }

        pub fn slash(self) -> DateFormatBuilder<AddComponent> {
            self.add_separator(Separator::Slash)
        }

        pub fn dot(self) -> DateFormatBuilder<AddComponent> {
            self.add_separator(Separator::Dot)
        }

        pub fn dash(self) -> DateFormatBuilder<AddComponent> {
            self.add_separator(Separator::Dash)
        }

        pub fn space(self) -> DateFormatBuilder<AddComponent> {
            self.add_separator(Separator::Space)
        }

        pub fn build(self) -> DateFormat {
            DateFormat {
                units: self.units.into_boxed_slice(),
            }
        }
    }
}
