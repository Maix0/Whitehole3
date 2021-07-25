use serenity::prelude::TypeMapKey;
pub struct DatabaseKey;

impl TypeMapKey for DatabaseKey {
    type Value = sqlx::PgPool;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Id(pub u64);

impl From<i64> for Id {
    fn from(id: i64) -> Self {
        Self(unsafe { std::mem::transmute(id) })
    }
}

impl<'q, DB: ::sqlx::Database> ::sqlx::encode::Encode<'q, DB> for Id
where
    i64: ::sqlx::encode::Encode<'q, DB>,
{
    fn encode_by_ref(
        &self,
        buf: &mut <DB as ::sqlx::database::HasArguments<'q>>::ArgumentBuffer,
    ) -> ::sqlx::encode::IsNull {
        <i64 as ::sqlx::encode::Encode<'q, DB>>::encode_by_ref(
            &unsafe { std::mem::transmute(self.0) },
            buf,
        )
    }
    fn produces(&self) -> Option<DB::TypeInfo> {
        <i64 as ::sqlx::encode::Encode<'q, DB>>::produces(&unsafe { std::mem::transmute(self.0) })
    }
    fn size_hint(&self) -> usize {
        <i64 as ::sqlx::encode::Encode<'q, DB>>::size_hint(&unsafe { std::mem::transmute(self.0) })
    }
}
impl<'r, DB: ::sqlx::Database> ::sqlx::decode::Decode<'r, DB> for Id
where
    i64: ::sqlx::decode::Decode<'r, DB>,
{
    fn decode(
        value: <DB as ::sqlx::database::HasValueRef<'r>>::ValueRef,
    ) -> ::std::result::Result<
        Self,
        ::std::boxed::Box<
            dyn ::std::error::Error + 'static + ::std::marker::Send + ::std::marker::Sync,
        >,
    > {
        <i64 as ::sqlx::decode::Decode<'r, DB>>::decode(value)
            .map(|e| unsafe { std::mem::transmute(e) })
            .map(Self)
    }
}
impl ::sqlx::Type<::sqlx::postgres::Postgres> for Id {
    fn type_info() -> ::sqlx::postgres::PgTypeInfo {
        <i64 as sqlx::Type<sqlx::postgres::Postgres>>::type_info()
    }
}
