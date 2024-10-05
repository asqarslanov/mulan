use std::fmt::{self, Debug, Formatter};

use strum::IntoStaticStr;

#[derive(IntoStaticStr)]
#[strum(serialize_all = "lowercase")]
enum Component {
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
enum Separator {
    #[strum(to_string = "/")]
    Slash,
    #[strum(to_string = ".")]
    Dot,
    #[strum(to_string = "-")]
    Dash,
    #[strum(to_string = " ")]
    Space,
}

enum Unit {
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

pub struct DateFormat(Box<[Unit]>);

impl Debug for DateFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DateFormat")
            .field(
                &self
                    .0
                    .iter()
                    .map(<&'static str>::from)
                    .collect::<Box<str>>(),
            )
            .finish()
    }
}

impl DateFormat {
    #[must_use]
    pub fn builder() -> AddComponent {
        const UNITS: usize = 5;
        AddComponent(Vec::with_capacity(UNITS))
    }
}

pub struct AddComponent(Vec<Unit>);

pub struct AddSeparator(Vec<Unit>);

impl AddComponent {
    pub fn yy(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Yy));
        AddSeparator(self.0)
    }

    pub fn yyyy(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Yyyy));
        AddSeparator(self.0)
    }

    pub fn m(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::M));
        AddSeparator(self.0)
    }

    pub fn mm(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Mm));
        AddSeparator(self.0)
    }

    pub fn mmm(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Mmm));
        AddSeparator(self.0)
    }

    pub fn mmmm(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Mmmm));
        AddSeparator(self.0)
    }

    pub fn d(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::D));
        AddSeparator(self.0)
    }

    pub fn dd(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Dd));
        AddSeparator(self.0)
    }

    pub fn ddd(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Ddd));
        AddSeparator(self.0)
    }

    pub fn dddd(mut self) -> AddSeparator {
        self.0.push(Unit::Component(Component::Dddd));
        AddSeparator(self.0)
    }
}

impl AddSeparator {
    pub fn build(self) -> DateFormat {
        DateFormat(self.0.into_boxed_slice())
    }

    pub fn slash(mut self) -> AddComponent {
        self.0.push(Unit::Separator(Separator::Slash));
        AddComponent(self.0)
    }

    pub fn dot(mut self) -> AddComponent {
        self.0.push(Unit::Separator(Separator::Dot));
        AddComponent(self.0)
    }

    pub fn dash(mut self) -> AddComponent {
        self.0.push(Unit::Separator(Separator::Dash));
        AddComponent(self.0)
    }

    pub fn space(mut self) -> AddComponent {
        self.0.push(Unit::Separator(Separator::Space));
        AddComponent(self.0)
    }
}
