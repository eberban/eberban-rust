/// Shorthand to write a Parser function.
#[macro_export]
macro_rules! rule {
    ($vis:vis $rule_name:ident , $in:ty => $out:ty , $body:expr) => {
        $vis fn $rule_name<S>() -> impl Parser<S, $in, $out, Error = Error> {
            $body
        }
    };
    ($vis:vis $rule_name:ident($($arg_name:ident : $arg_type:ty)*) , $in:ty => $out:ty , $body:expr) => {
        $vis fn $rule_name<S>($($arg_name : $arg_type)*) -> impl Parser<S, $in, $out, Error = Error> {
            $body
        }
    };
}

#[macro_export]
macro_rules! rules {
    ($(
        $(#[$meta:meta])*
        $vis:vis
        $rule_name:ident
        $(
            (
                $($arg_name:ident : $arg_type:ty)*
            )
        )?
        [$in:ty => $out:ty]
        =
        $body:expr
        ;
    )+) => {
        $(
            $(#[$meta])*
            $vis
            fn $rule_name<S>(
                $(
                    $($arg_name : $arg_type)*
                )?
            ) -> impl Parser<S, $in, $out, Error = Error> {
                $body
            }
        )+
    };
}
