use crate::util;
use jimage_rs::JImage;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader, Cursor, Read, Seek};
use std::path::{self, Path};
use std::sync::{Arc, Mutex, RwLock};
use zip::ZipArchive;

lazy_static! {
    static ref CPM: RwLock<ClassPathManager> = { RwLock::new(ClassPathManager::new()) };
}

pub fn init() {
    lazy_static::initialize(&CPM);
}

pub fn find_class(name: &str) -> Result<ClassPathResult, io::Error> {
    let cpm = CPM.read().unwrap();
    cpm.search_class(name)
}

pub fn add_path(path: &str) {
    let mut cpm = CPM.write().unwrap();
    cpm.add_class_path(path);
}

pub fn add_paths(path: &str) {
    let mut cpm = CPM.write().unwrap();
    cpm.add_class_paths(path);
}

/// 添加 JImage（JDK 9+ modules 文件）作为引导类路径源。
pub fn add_boot_jimage(path: &str) {
    let mut cpm = CPM.write().unwrap();
    if let Err(e) = cpm.add_jimage_path(path) {
        warn!("Failed to add boot JImage path '{}': {}", path, e);
    } else {
        info!("Boot JImage loaded from: {}", path);
    }
}

#[derive(Debug)]
pub struct ClassPathResult(pub String, pub Vec<u8>);

type ZipRef = Arc<Mutex<Box<ZipArchive<File>>>>;

/// JImage 源，包含 JImage 实例和后缀索引。
/// 索引将短路径（如 "java/lang/String"）映射到完整 JImage 资源路径
///（如 "/java.base/java/lang/String.class"）。
struct JImageSource {
    jimage: JImage,
    index: HashMap<String, String>,
}

impl JImageSource {
    fn new(jimage: JImage) -> Self {
        let mut index = HashMap::new();
        for rn in jimage.resource_names_iter() {
            if let Ok(rn) = rn {
                let module = rn.module.to_string();
                let parent = rn.parent.to_string();
                let base = rn.base.to_string();
                let extension = rn.extension.to_string();
                if extension == "class" {
                    // 索引 key: "java/lang/String"（不含模块前缀）
                    // search_class 传入的是 "java/lang/String" 格式
                    let key = if parent.is_empty() {
                        base.clone()
                    } else {
                        format!("{}/{}", parent, base)
                    };
                    // JImage find_resource 需要的完整路径: "/java.base/java/lang/String.class"
                    let full_path = if parent.is_empty() {
                        format!("/{}/{}.{}", module, base, extension)
                    } else {
                        format!("/{}/{}/{}.{}", module, parent, base, extension)
                    };
                    // 同一个 key 可能被多个模块提供（如 jdk.internal.*），
                    // 取第一个即可（bootstrap 类通常在 java.base）
                    index.entry(key).or_insert(full_path);
                }
            }
        }
        Self { jimage, index }
    }
}

enum ClassSource {
    DIR,
    JAR(ZipRef),
    JIMAGE(JImageSource),
}

struct ClassPathEntry(ClassSource, String);

struct ClassPathManager {
    runtime_class_path: Vec<ClassPathEntry>,
}

impl ClassPathManager {
    fn new() -> Self {
        Self {
            runtime_class_path: vec![],
        }
    }

    pub fn add_class_path(&mut self, path: &str) -> Result<(), io::Error> {
        let p = Path::new(path);
        if p.is_dir() {
            self.runtime_class_path
                .push(ClassPathEntry(ClassSource::DIR, path.to_string()));
        } else {
            let f = File::open(p)?;
            let mut z = ZipArchive::new(f)?;
            let handle = Arc::new(Mutex::new(Box::new(z)));
            self.runtime_class_path
                .push(ClassPathEntry(ClassSource::JAR(handle), path.to_string()));
        }

        Ok(())
    }

    pub fn add_class_paths(&mut self, path: &str) {
        path.split(util::PATH_SEP).for_each(|p| {
            if let Err(e) = self.add_class_path(p) {
                error!("add class path error, path={}, e={:?}", p, e)
            }
        });
    }

    /// 添加 JImage（JDK 9+ modules 文件）作为类路径源。
    pub fn add_jimage_path(&mut self, path: &str) -> Result<(), String> {
        let jimage = JImage::open(path).map_err(|e| format!("Failed to open JImage: {}", e))?;
        let source = JImageSource::new(jimage);
        self.runtime_class_path.push(ClassPathEntry(
            ClassSource::JIMAGE(source),
            path.to_string(),
        ));
        Ok(())
    }

    pub fn search_class(&self, name: &str) -> Result<ClassPathResult, io::Error> {
        let name = name.replace("/", util::FILE_SEP);
        let name = name.replace(".", util::FILE_SEP);

        trace!("search_class: {}", name);

        for it in self.runtime_class_path.iter() {
            match &it.0 {
                ClassSource::DIR => {
                    let mut p = String::from(&it.1);
                    p.push_str(util::FILE_SEP);
                    p.push_str(&name);
                    p.push_str(".class");
                    if let Ok(data) = std::fs::read(&p) {
                        return Ok(ClassPathResult(p, data));
                    }
                }

                ClassSource::JAR(handle) => {
                    let mut p = String::from(&name);
                    p.push_str(".class");

                    let mut handle = handle.lock().unwrap();
                    let mut zf = handle.by_name(&p);

                    if let Ok(mut zf) = zf {
                        let mut v = Vec::with_capacity(zf.size() as usize);
                        let r = zf.read_to_end(&mut v);
                        debug_assert!(r.is_ok());
                        return Ok(ClassPathResult(it.1.clone(), v));
                    }
                }

                ClassSource::JIMAGE(src) => {
                    // JImage 索引使用 / 分隔符，需要还原原始格式
                    let jimage_key = name.replace(util::FILE_SEP, "/");
                    if let Some(full_path) = src.index.get(&jimage_key) {
                        match src.jimage.find_resource(full_path) {
                            Ok(Some(data)) => {
                                return Ok(ClassPathResult(it.1.clone(), data.into_owned()));
                            }
                            Ok(None) => {
                                // 哈希冲突或索引不一致，继续下一个 entry
                                trace!(
                                    "JImage index mismatch for {}, full_path={}",
                                    name,
                                    full_path
                                );
                            }
                            Err(e) => {
                                error!("JImage find_resource error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Search class failed: {}", name),
        ))
    }

    pub fn size(&self) -> usize {
        self.runtime_class_path.len()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn t_basic_zip() {
        let f = "test/class_path_test.jar";
        let f = super::File::open(f).unwrap();
        let mut za = super::ZipArchive::new(f).unwrap();

        for i in 0..za.len() {
            let mut zf = za.by_index(i).unwrap();
            println!("{}", zf.name());
        }
    }

    #[test]
    fn t_replace_all() {
        let class = "java.lang.String";
        assert_eq!(class.replace(".", "/"), "java/lang/String");
    }

    #[test]
    fn t_add_cls_path() {
        let mut cpm = super::ClassPathManager::new();
        assert!(cpm.add_class_path("test/").is_ok());
        assert!(cpm.add_class_path("test_no_exist/").is_err());
        assert!(cpm
            .add_class_path("test/classloader/class_path_test.jar")
            .is_ok());
        assert!(cpm
            .add_class_path("test/classloader/class_path_test_no_exist.jar")
            .is_err());
    }

    #[test]
    fn t_add_cls_paths() {
        let mut cpm = super::ClassPathManager::new();
        cpm.add_class_paths("test/:test/classloader/class_path_test.jar");
        assert_eq!(2, cpm.size());
    }

    #[test]
    fn t_search_cls() {
        let mut cpm = super::ClassPathManager::new();
        let _ = cpm.add_class_path("test/classloader/class_path_test.jar");
        assert!(cpm.search_class("Foo").is_ok());
    }

    #[test]
    fn t_search_cls2() {
        let mut cpm = super::ClassPathManager::new();
        cpm.add_class_paths("test/classloader/class_path_test.jar");
        assert!(cpm.search_class("Sample").is_err());
        assert!(cpm.search_class("Foo").is_ok());
    }

    /// 测试 JImage 索引构建和类搜索。
    /// 使用系统 JDK 的 modules 文件验证 java.lang.String 可被找到。
    #[test]
    fn t_search_jimage() {
        let jhome = if cfg!(target_os = "macos") {
            // macOS: 尝试 /usr/libexec/java_home
            std::process::Command::new("/usr/libexec/java_home")
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
        } else {
            std::env::var("JAVA_HOME").ok()
        };

        let modules_path = match jhome {
            Some(home) => format!("{}/lib/modules", home),
            None => return, // 没有 JDK 9+，跳过测试
        };

        if !std::path::Path::new(&modules_path).exists() {
            return;
        }

        let mut cpm = super::ClassPathManager::new();
        assert!(cpm.add_jimage_path(&modules_path).is_ok());
        assert_eq!(cpm.size(), 1);

        // 验证 java/lang/String 可被找到
        let result = cpm.search_class("java/lang/String");
        assert!(result.is_ok(), "Should find java/lang/String: {:?}", result);
        let bytes = result.unwrap().1;
        assert!(
            bytes.len() > 1000,
            "String.class should be large, got {} bytes",
            bytes.len()
        );
        // 验证是有效的 class 文件（magic number: CAFEBABE）
        assert_eq!(&bytes[0..4], &[0xCA, 0xFE, 0xBA, 0xBE]);

        // 验证 java/lang/Object 可被找到
        let result = cpm.search_class("java/lang/Object");
        assert!(result.is_ok());

        // 验证不存在的类返回错误
        assert!(cpm.search_class("com/nonexistent/Foo").is_err());
    }
}
