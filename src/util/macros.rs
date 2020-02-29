#[macro_export]
macro_rules! def_sync_ref {
    ($name:ident, $t:ty) => {
        pub type $name = std::sync::Arc<std::sync::RwLock<Box<$t>>>;
    };
}

#[macro_export]
macro_rules! def_ref {
    ($name:ident, $t:ty) => {
        pub type $name = std::sync::Arc<Box<$t>>;
    };
}

#[macro_export]
macro_rules! def_ptr {
    ($name:ident, $t:ty) => {
        pub type $name = Box<$t>;
    };
}

#[macro_export]
macro_rules! new_sync_ref {
    ($name:ident) => {
        std::sync::Arc::new(std::sync::RwLock::new(Box::new($name)));
    };
}

#[macro_export]
macro_rules! new_ref {
    ($name:ident) => {
        std::sync::Arc::new(Box::new($name));
    };
}
