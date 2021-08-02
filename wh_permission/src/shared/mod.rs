pub mod role_permission;
pub mod user_permission;

#[macro_export]
macro_rules! check_permission {
    ($struct_name:ident, $permission:literal) => {
        const $struct_name: serenity::framework::standard::Check =
            serenity::framework::standard::Check {
                function: |ctx, msg, _, _| {
                    $crate::shared::user_permission::check_permission(ctx, msg, $permission)
                },
                name: $permission,
                display_in_help: true,
                check_in_help: true,
            };
    };
}
