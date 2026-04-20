# Slot-Based Heap

## 核心思想

用 `u32` slot_id 代替裸指针引用堆对象，实现零 unsafe 的对象访问和 precise GC 基础。

## 架构

```rust
// Oop enum：u32 索引代替 Arc<OopPtr>
enum Oop {
    Int(i32), Long(i64), Float(f32), Double(f64),
    ConstUtf8(BytesRef), Null,
    Ref(u32),  // slot_id
}

// Heap：Vec 按 slot_id 索引
struct Heap {
    slots: Vec<Option<Arc<RwLock<RefKindDesc>>>>,
    free_list: Vec<u32>,
}

// 访问模式
oop::with_heap(|heap| {
    let desc = heap.get(slot_id);
    let guard = desc.read().unwrap();
    guard.v.extract_inst().class.clone()
});
```

## 实现原理

### 分配

```
alloc(desc) → slot_id
```

1. 优先从 `free_list` 取空闲 slot_id，复用已释放位置
2. `free_list` 为空时，`slots.push()` 扩展
3. 返回 `u32`，写入 `Oop::Ref(slot_id)`

### 访问

`Oop::Ref(slot_id)` 持有的是索引，实际数据在 `Heap.slots[slot_id]` 中。通过 `Arc<RwLock<RefKindDesc>>` 访问：

- `Arc::clone`：线程安全的共享所有权
- `RwLock::read()`：读并发
- `RwLock::write()`：写互斥

### 释放

```
free(slot_id) → slots[slot_id] = None, free_list.push(slot_id)
```

slot 内容清空，slot_id 回收到 `free_list`。下次分配时可复用。

### Precise GC 基础

`Oop` 的枚举变体天然区分引用和值：

| 变体 | 含义 | GC 处理 |
|------|------|---------|
| `Ref(u32)` | 堆对象引用 | 标记该 slot |
| `Int/Long/Float/Double` | 基本类型 | 跳过 |
| `Null` | 空引用 | 跳过 |
| `ConstUtf8` | 类文件元数据 | 跳过 |

不需要像 conservative GC 那样猜测栈上的值是不是指针——类型就是答案。

### GC 压缩

`slot_id` 是间接层。GC 移动对象时只需更新 slot 内部的 `Arc`，外部所有 `Oop::Ref(slot_id)` 自动指向新位置，无需修改。

## 缺陷

### 1. 每次访问都要获取锁

```rust
// 读一个字段：slot_id → Arc clone → RwLock read guard → 取值
// 2-3 倍间接开销 vs 裸指针直接解引用
```

### 2. 内存开销

每个对象额外携带 `Arc`（2 个 usize）+ `RwLock`（30-50 字节）。空对象实际占用 ~80-100 字节，生产 JVM 仅 ~16 字节。

### 3. Vec 无限增长

分配 100 万对象后释放 99 万，Vec 容量不变，`free_list` 只回收 slot_id 不回收内存。

### 4. ABA 问题

slot 被 free 后重新分配给新对象，持有旧 `Oop::Ref(slot_id)` 的代码会静默指向不同对象。影响 `WeakReference` 等场景。

### 5. 不可跨进程共享

slot_id 是进程内 `Vec` 索引，无法 mmap 或跨进程传递。

## 当前状态

提供分配和 slot 复用（`free_list`），尚无 GC 扫描。是 Phase 8 mark-sweep-compact 的基础。

## 未来改进方向

### Phase 8: Mark-Sweep-Compact

- 遍历 `slots` 找出所有活对象，清理死对象
- Compact 阶段：合并空闲区域，减少 Vec 容量
- 解决缺陷 3（Vec 增长）

### Safepoint 机制

- GC 期间暂停所有线程，消除运行中的锁竞争
- 访问路径从 `RwLock` 降级为裸引用（线程安全暂停期间）
- 解决缺陷 1（锁开销）

### 细粒度内存布局

- 用 `UnsafeCell` 或 `ManuallyDrop` + 自定义 allocator 替代 `Arc<RwLock<T>>`
- 对象紧凑排列，去除 Arc/RwLock 元数据
- 解决缺陷 2（内存开销）

### Slot Version Tag

- `slot_id` 改为 `(index, version)` 组合
- 每次 slot 重分配时递增 version
- 旧引用 version 不匹配时自动失效
- 解决缺陷 4（ABA 问题）
