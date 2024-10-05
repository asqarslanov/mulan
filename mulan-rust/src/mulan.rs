pub mod context;
pub mod options;

#[derive(Clone, Copy)]
pub enum Locale {
    EnUs,
}

// static mut CONTEXT_OPTIONS: OnceLock<HashMap<Vec<String>, Locale>> = OnceLock::new();

// fn context_options_mut() -> &'static HashMap<Vec<String>, Locale> {
//     CONTEXT_OPTIONS.get_mut().unwrap()
// }

// fn context_options() -> &'static HashMap<Vec<String>, Locale> {
//     CONTEXT_OPTIONS.get_or_init(|| HashMap::new())
// }

// struct Context {
//     context_path: Vec<String>,
// }

// impl Context {
//     fn t(self) -> private::T {
//         private::context(self.context_path)
//     }

//     fn context_path(&self) -> &[String] {
//         &self.context_path
//     }

//     fn current_locale(&self) -> Locale {
//         *context_options().get(self.context_path()).unwrap()
//     }

//     fn set_locale(&self, locale: Locale) {
//         context_options().insert(self.context_path.clone(), locale);
//     }
// }

// pub fn context(context_path: Vec<String>) -> () {
//     todo!()
// }

// mod private {
//     pub fn context(context_path: Vec<String>) -> T {
//         T {
//             context_path_: context_path,
//         }
//     }

//     pub struct T {
//         context_path_: Vec<String>,
//     }
// }
