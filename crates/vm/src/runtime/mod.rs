#![allow(unused)]

pub use class_loader::{require_class, require_class2, require_class3, ClassLoader};
pub use class_path_manager::{
    add_boot_jimage, add_path as add_class_path, add_paths as add_class_paths,
    find_class as find_class_in_classpath, ClassPathResult,
};
pub use constant_pool::ConstantPoolCache;
pub use consts::THREAD_MAX_STACK_FRAMES;
pub use dataarea::DataArea;
pub use frame::Frame;
pub use interp::Interp;
pub use invoke::JavaCall;
pub use slot::Slot;
pub use sys_dic::{find as sys_dic_find, put as sys_dic_put};
pub use thread::JavaThread;

mod class_loader;
mod class_path_manager;
pub mod cmp;
mod constant_pool;
mod consts;
mod dataarea;
pub mod exception;
mod frame;
mod init_vm;
pub mod interp;
pub mod invoke;
pub mod jit;
mod local;
pub mod method;
mod slot;
mod stack;
mod sys_dic;
pub mod thread;
pub mod vm;

pub fn init() {
    crate::oop::init_vm_state();
    sys_dic::init();
    class_path_manager::init();

    // 自动检测并加载 JDK 9+ 的 JImage modules 文件
    init_boot_jimage();

    // JIT temporarily disabled for stability
    // if let Err(e) = jit::init() {
    //     warn!("JIT compiler initialization failed: {}, JIT disabled", e);
    // } else {
    //     info!("JIT compiler initialized");
    // }
}

/// 自动检测并添加 JDK 9+ 的引导类路径（$JAVA_HOME/lib/modules）。
fn init_boot_jimage() {
    // 1. 优先使用 JAVA_HOME 环境变量
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let modules_path = format!("{}/lib/modules", java_home);
        if std::path::Path::new(&modules_path).exists() {
            class_path_manager::add_boot_jimage(&modules_path);
            return;
        }
    }

    // 2. macOS 专用回退：/usr/libexec/java_home
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("/usr/libexec/java_home").output() {
            if output.status.success() {
                let java_home = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let modules_path = format!("{}/lib/modules", java_home);
                if std::path::Path::new(&modules_path).exists() {
                    class_path_manager::add_boot_jimage(&modules_path);
                    return;
                }
            }
        }
    }

    // 3. Linux 常见 JDK 路径回退
    let candidates = [
        "/usr/lib/jvm/default/lib/modules",
        "/usr/lib/jvm/java/lib/modules",
        "/usr/lib64/jvm/default/lib/modules",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            class_path_manager::add_boot_jimage(path);
            return;
        }
    }

    warn!("No JImage modules file found. JDK 9+ classes will not be available.");
}
