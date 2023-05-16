#[macro_export]
macro_rules! response {
    ($name:tt,$code:expr,$message:expr,$($key:tt:$value:expr),*) => {
        return Ok(Response::new($name{
            $(
                $key : $value,
            )*
            code : $code,
            message: $message
        }))

    };
}

#[macro_export]
macro_rules! success {
    ($name:tt,$($key:tt:$value:expr),*) => {
        response!($name,0,"success".into(),$($key:$value)*)
    };
}

#[macro_export]
macro_rules! bad_request {
    ($name:tt,$message:expr,$($key:tt:$value:expr),*) => {
        response!($name,400,$message,$($key:$value)*)
    };
}

#[macro_export]
macro_rules! server_err {
    ($name:tt,$error:expr,$($key:tt:$value:expr),*) => {
        response!($name,500,format!("server error:{:?}",$error),$($key:$value)*)
    };
}

