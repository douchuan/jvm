use crate::oop::{self, consts, InstOopDesc, Oop};
use crate::runtime::{self, init_vm, require_class3, JavaCall};
use crate::types::{ClassRef, FrameRef, JavaThreadRef, MethodIdRef};
use crate::util::{self, new_field_id, new_method_id};
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

lazy_static! {
    static ref NATIVE_THREAD_POOL: Mutex<HashMap<std::thread::ThreadId, JavaThreadRef>> =
        { Mutex::new(HashMap::new()) };
}

pub struct JavaThread {
    pub frames: Vec<FrameRef>,
    in_safe_point: bool,

    pub java_thread_obj: Option<Oop>,
    ex: Option<Oop>,
}

pub struct JavaMainThread {
    pub class: String,
    pub args: Option<Vec<String>>,
    dispatch_uncaught_exception_called: bool,
}

impl JavaThread {
    pub fn new() -> JavaThreadRef {
        let t = Self {
            frames: Vec::new(),
            in_safe_point: false,

            java_thread_obj: None,
            ex: None,
        };
        Arc::new(RwLock::new(Box::new(t)))
    }

    pub fn set_java_thread_obj(&mut self, obj: Oop) {
        self.java_thread_obj = Some(obj);
    }
}

//exception
impl JavaThread {
    pub fn set_ex(&mut self, ex: Oop) {
        self.ex = Some(ex);
    }

    pub fn is_meet_ex(&self) -> bool {
        self.ex.is_some()
    }

    pub fn take_ex(&mut self) -> Option<Oop> {
        self.ex.take()
    }
}

impl JavaMainThread {
    pub fn new(class: String, args: Option<Vec<String>>) -> Self {
        Self {
            class,
            args,
            dispatch_uncaught_exception_called: false,
        }
    }

    pub fn run(&mut self) {
        let jt = JavaThread::new();

        info!("init vm start...");
        init_vm::initialize_jvm(jt.clone());
        info!("init vm end");

        let main_class = oop::class::load_and_init(jt.clone(), self.class.as_bytes());

        let mir = {
            let cls = main_class.read().unwrap();

            /*
            path info should be included in "--cp", and avoid same class load 2
            times, otherwise, "<clinit>" invoked 2 times.

            For example:
              "MyFile.java":
                private static File gf = newFile();

            if allowed, as follows:
              "cargo run -- --cp $JDK:$MY_TEST test/with_package/my.ns.HelloWorld"
            will cause "<clinit>" invoked 2 times, "newFile()" invoked 2 times,
            maybe create 2 files.

            should be like this:
              "cargo run -- --cp $JDK:$MY_TEST:test/with_package my.ns.HelloWorld"
            */
            if self.class.as_bytes() != cls.name.as_slice() {
                panic!("Error: Could not find or load main class {}", self.class);
            }

            cls.get_static_method(b"main", b"([Ljava/lang/String;)V")
        };

        match mir {
            Ok(mir) => {
                let arg = self.build_main_arg(jt.clone());
                let area = runtime::DataArea::new(0, 1);
                {
                    area.write().unwrap().stack.push_ref(arg);
                }
                match JavaCall::new(jt.clone(), &area, mir) {
                    Ok(mut jc) => jc.invoke(jt.clone(), Some(&area), true),
                    _ => unreachable!(),
                }
            }
            _ => unimplemented!(),
        }

        let is_ex = { jt.read().unwrap().ex.is_some() };
        if is_ex {
            self.uncaught_ex(jt.clone(), main_class);
        }
    }
}

impl JavaMainThread {
    fn build_main_arg(&self, jt: JavaThreadRef) -> Oop {
        let args = match &self.args {
            Some(args) => args
                .iter()
                .map(|it| util::oop::new_java_lang_string2(jt.clone(), it))
                .collect(),
            None => vec![],
        };

        //build ArrayOopDesc
        let ary_str_class = runtime::require_class3(None, b"[Ljava/lang/String;").unwrap();
        Oop::new_ref_ary2(ary_str_class, args)
    }

    fn uncaught_ex(&mut self, jt: JavaThreadRef, main_cls: ClassRef) {
        if self.dispatch_uncaught_exception_called {
            self.uncaught_ex_internal(jt);
        } else {
            self.dispatch_uncaught_exception_called = true;
            self.call_dispatch_uncaught_exception(jt, main_cls);
        }
    }

    fn call_dispatch_uncaught_exception(&mut self, jt: JavaThreadRef, main_cls: ClassRef) {
        let v = {
            let jt = jt.read().unwrap();
            jt.java_thread_obj.clone()
        };
        match v {
            Some(v) => {
                let cls = {
                    let v = util::oop::extract_ref(&v);
                    let v = v.read().unwrap();
                    match &v.v {
                        oop::RefKind::Inst(inst) => inst.class.clone(),
                        _ => unreachable!(),
                    }
                };

                let mir = {
                    let cls = cls.read().unwrap();
                    cls.get_this_class_method(
                        b"dispatchUncaughtException",
                        b"(Ljava/lang/Throwable;)V",
                    )
                };

                match mir {
                    Ok(mir) => {
                        let ex = {
                            let mut jt = jt.write().unwrap();
                            jt.take_ex().unwrap()
                        };
                        let args = vec![v.clone(), ex];
                        let mut jc = JavaCall::new_with_args(jt.clone(), mir, args);
                        let area = runtime::DataArea::new(0, 0);
                        jc.invoke(jt, Some(&area), false);
                    }
                    _ => self.uncaught_ex_internal(jt),
                }
            }

            None => self.uncaught_ex_internal(jt),
        }
    }

    fn uncaught_ex_internal(&mut self, jt: JavaThreadRef) {
        let ex = {
            let mut jt = jt.write().unwrap();
            jt.take_ex().unwrap()
        };

        let cls = {
            match &ex {
                Oop::Ref(v) => {
                    let v = v.read().unwrap();
                    match &v.v {
                        oop::RefKind::Inst(inst) => inst.class.clone(),
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            }
        };
        let detail_message = {
            let v = {
                let cls = cls.read().unwrap();
                let id = cls.get_field_id(b"detailMessage", b"Ljava/lang/String;", false);
                cls.get_field_value(&ex, id)
            };

            util::oop::extract_str(&v)
        };
        let name = {
            let cls = cls.read().unwrap();
            cls.name.clone()
        };

        let cls_name = String::from_utf8_lossy(name.as_slice());
        error!("Name={}, detailMessage={}", cls_name, detail_message);
    }
}
