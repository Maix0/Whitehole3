//! This crate allow the use of the [`Fluent Project`] as a proc macro.
//!
//! This allow a compile time version of fluent, thus enabling the use of `format_args!()` type macro such as `println!()` or `write!()`.
//! To use this macro you need to provide the FTL file path using the `FLUENT_FILE` evironement variable.
//!
//! # Example
//! ```no_run
//! println!(fluent!(message_id)) // This output whatever `message_id` resolve to
//! ```    
//! [`Fluent Project`]: https://projectfluent.org/

extern crate dotenv;
extern crate fluent;
extern crate intl_memoizer;
extern crate once_cell;
extern crate proc_macro;
extern crate quote;

use once_cell::sync::Lazy;
use quote::quote;

static FLUENT_BUNDLE: Lazy<
    Result<
        fluent::bundle::FluentBundle<
            fluent::FluentResource,
            intl_memoizer::concurrent::IntlLangMemoizer,
        >,
        Vec<String>,
    >,
> = Lazy::new(|| {
    let mut bundle = fluent::bundle::FluentBundle::new_concurrent(Default::default());
    dotenv::dotenv().map_err(|e| vec![format!("Dotenv Error: {}", e)])?;
    #[cfg(not(doc))]
    let resources_file =
        std::env::var("FLUENT_FILE").map_err(|e| vec![format!("Env Error: {}", e)])?;
    #[cfg(doc)]
    let resources_file = "CARGODOC";
    let file =
        std::fs::read_to_string(resources_file).map_err(|e| vec![format!("Read Error: {}", e)])?;
    let ressources = fluent::FluentResource::try_new(file).map_err(|e| {
        e.1.iter()
            .map(|e| format!("Fuent Error: {}", e))
            .collect::<Vec<String>>()
    })?;
    bundle.add_resource_overriding(ressources);
    Ok(bundle)
});

#[proc_macro]
/// The macro of this crate
/// This will resolve at compile time whatever message id you have inputed.
/// # Example
/// ```no_run
/// let hello_world: &'static str = fluent!(hello_world);
/// ```
/// You can also use the macro for string litteral inside of `println!()`;
/// ```no_run
/// println!(fluent!(hello), name="world");
/// ```
/// this is an example of the fluent.ftl file:
/// ```text
/// # this is a comment
/// hello_world=hello world
/// hello = hello {"{name}"} # the `{"..."}` is used to escape the `{name}`
/// ```
/// for more information check [here](https://projectfluent.org/fluent/guide/text.html)
pub fn fluent(token: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let token_str = token.to_string();
    if let Err(msg) = FLUENT_BUNDLE.as_ref() {
        return quote!(#(compile_error!(#msg))*).into();
    }
    let message = FLUENT_BUNDLE.as_ref().unwrap().get_message(&token_str);
    if message.is_none() {
        return quote!(compile_error!("The given fluent id isn't found!")).into();
    }
    let message = message.unwrap();

    let message = if let Some(value) = message.value() {
        let mut err = vec![];
        Some(
            FLUENT_BUNDLE
                .as_ref()
                .unwrap()
                .format_pattern(value, None, &mut err),
        )
    } else {
        return quote!(compile_error!("fluent_const doesn't support patterns")).into();
    };
    let message = message.unwrap();
    quote!(#message).into()
}
