# JDK 9+ JRT/JImage 格式变化

## 背景

JDK 8 及之前，Java 标准库（`java.lang.*`、`java.util.*` 等）打包在 `lib/rt.jar` 中，
这是一个标准的 ZIP/JAR 文件，可以直接用 ZIP 库读取。

JDK 9 引入了 [JEP 220: Modular Run-Time Images](https://openjdk.org/jeps/220)，
`rt.jar` 被 `$JAVA_HOME/lib/modules` 文件替代。该文件使用 **JImage** 格式，
不是标准 ZIP，不能直接用 ZIP 库读取。

## 为什么改变

| 指标 | rt.jar (JDK 8) | modules / JImage (JDK 9+) |
|------|----------------|---------------------------|
| 文件格式 | ZIP/JAR | 自定义二进制格式 |
| 大小 | ~60MB | ~40MB（更紧凑） |
| 启动速度 | 较慢（ZIP 目录解析慢） | 更快（内存映射 + 哈希索引） |
| 模块化 | 无 | 支持 JPMS 模块系统 |

## JImage 文件结构

```
/------------------------------\
|          Header              |  (固定大小: 28 字节)
|------------------------------|
|       Index Tables:          |
|  - Redirect Table            |  (table_length * 4 字节)
|  - Offsets Table             |  (table_length * 4 字节)
|  - Location Attributes Table |  locations_bytes
|------------------------------|
|         String Table         |  strings_bytes (null 分隔的字符串表)
|------------------------------|
|                              |
|       Resource Data Blob     |  实际的 class/资源数据（部分 zlib 压缩）
|                              |
\------------------------------/
```

### Header (28 bytes)

| 偏移 | 大小 | 含义 |
|------|------|------|
| 0x00 | 4 | Magic: `0x4A494D47` ("JIMG") |
| 0x04 | 4 | 版本 |
| 0x08 | 4 | 条目数 (items_count) |
| 0x0C | 4 | 字符串表偏移 |
| 0x10 | 4 | 字符串表大小 |
| 0x14 | 4 | 属性表大小 |
| 0x18 | 4 | table_length |

### 资源查找流程

1. 用哈希函数计算资源路径的 hash：`hash(name) % items_count`
2. 通过 redirect table 找到 offset index
3. 通过 offset table 找到 attribute 索引
4. 解析 attribute 获取：module、parent、base、extension、offset、compressed/uncompressed size
5. 从 data blob 区域读取资源数据

资源路径格式：`/{module}/{parent}/{base}.{extension}`
例如：`/java.base/java/lang/String.class`

### 压缩

部分资源使用 zlib 压缩（属性中 `COMPRESSED` size > 0 表示压缩）。
资源头部包含 decompressor 名称（`"zip"` 即 zlib）。

## 对 JVM 实现的影响

### JDK 8 路径

```
lib/rt.jar  →  ZIP 读取  →  java/lang/String.class
```

### JDK 9+ 路径

```
lib/modules  →  JImage 解析  →  /java.base/java/lang/String.class
```

关键区别：
- 需要解析 JImage 二进制格式（非 ZIP）
- 资源路径包含模块名前缀（如 `java.base/`）
- 部分资源是 zlib 压缩的

## 参考实现

### Rust 库

| Crate | 版本 | 说明 |
|-------|------|------|
| [jimage-rs](https://crates.io/crates/jimage-rs) | 0.0.4 | 纯 Rust 实现，兼容 Rust 1.92+ |
| [ristretto_jimage](https://crates.io/crates/ristretto_jimage) | 0.30.0 | 需要 Rust 1.94+ |
| [jimage](https://github.com/MaulingMonkey/jimage) | 0.1.0 | FFI 绑定，依赖 jimage.dll/so |

### OpenJDK 源码

- Java 端：`src/java.base/share/classes/jdk/internal/jimage/`
  - `ImageHeader.java` — 文件头解析
  - `ImageReader.java` — 主读取器
  - `BasicImageReader.java` — 核心读取逻辑
  - `ImageLocation.java` — 资源位置结构
  - `jdk/internal/jrtfs/` — `jrt:` 文件系统实现
- Native 端：`src/java.base/share/native/libjimage/`
  - `imageFile.hpp` / `imageFile.cpp`

## 相关格式

| 格式 | 文件位置 | 用途 |
|------|----------|------|
| **JIMAGE** | `lib/modules` | JVM 运行时类加载的容器 |
| **JMOD** | `lib/jmods/*.jmod` | 模块打包格式（含 native 代码、配置），用于 jlink |
| **Modular JAR** | 用户 jar | 标准 JAR + module-info.class，向后兼容 JDK 8 |

## 注意事项

- JImage 格式**未公开规范**（[JDK-8061971](https://bugs.openjdk.org/browse/JDK-8061971)），
  官方声明格式可能随时变化
- 工具不应直接读取 jimage 文件，应使用 `jrt:` NIO FileSystem 或 `jimage` CLI 工具
- 但作为 JVM 实现，我们需要直接解析该格式以加载标准库类

## CLI 工具

```bash
# 列出 modules 文件中的资源
jimage list $JAVA_HOME/lib/modules

# 提取到目录
jimage extract --dir=/tmp/modules $JAVA_HOME/lib/modules
```
