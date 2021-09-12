macro_rules! define_error {
    (
    pub enum Error {
        $(
            $variant:ident = { description: $description:literal, code: $code:literal $(,)?}
        ),*
        $(,)?
    }
    ) => {
        #[derive(Clone, Copy)]
        pub enum Error {
            $(
                $variant,
            )*
        }

        impl Error {
            fn description(self, description: Option<String>) -> RspErr {
                match self {
                $(
                    Self::$variant => {
                        let description = description.unwrap_or_else(|| $description.into());
                        RspErr {description, code: $code}
                    }
                )*
                }
            }
            fn default_err(self) -> RspErr {
                self.description(None)
            }
        }
    };
}

define_error!(pub enum Error {
    
});


#[derive(Clone,Debug)]
pub struct RspErr{
    pub code: usize,
    pub description: String
}

impl std::cmp::PartialEq for RspErr {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}


#[derive(Clone,Debug)]
pub enum RspData<T> {
    Ok(T),
    Err(RspErr)
}