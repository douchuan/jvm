#[macro_export]
macro_rules! def_sync_ref {
    ($name:ident, $t:ty) => {
        pub type $name = std::sync::Arc<std::sync::Mutex<Box<$t>>>;
    };
}

#[macro_export]
macro_rules! def_ref {
    ($name:ident, $t:ty) => {
        pub type $name = std::sync::Arc<Box<$t>>;
    };
}

#[macro_export]
macro_rules! new_sync_ref {
    ($name:ident) => {
        std::sync::Arc::new(std::sync::Mutex::new(Box::new($name)));
    };
}

#[macro_export]
macro_rules! new_ref {
    ($name:ident) => {
        std::sync::Arc::new(Box::new($name));
    };
}

/*
#[macro_export]
macro_rules! new_id_ref {
    ($cls:ident, $method:ident, $desc:ident) => {
        let v = vec![$cls, $method, $desc].join(util);
    }
}
*/
