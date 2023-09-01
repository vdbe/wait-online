macro_rules! libc_bitflags {
    (
        $(#[$outer:meta])*
        $Visibility:vis enum $Flags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                $Flag:ident $(as $cast:ty)*;
            )+
        }
    ) => {
        $(#[$outer])*
        $Visibility enum $Flags {}

        impl $Flags {
            $(
                $(#[$inner $($args)*])*
                $Visibility const $Flag: $T = libc::$Flag $(as $cast)*;
            )+
        }
    };
}
