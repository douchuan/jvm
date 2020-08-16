use crate::oop::{self, Class, Oop, OopRef};
use crate::runtime::thread::thread_pool;
use crate::runtime::{self, init_vm, vm, DataArea, JavaCall, JavaThread};
use crate::types::{ClassRef, FrameRef, JavaThreadRef, MethodIdRef};
use crate::{new_br, util};
use std::borrow::Borrow;

pub struct MainThread {
    pub class: String,
    pub args: Option<Vec<String>>,
    dispatch_uncaught_exception_called: bool,
}

impl MainThread {
    pub fn new(class: String, args: Option<Vec<String>>) -> Self {
        Self {
            class,
            args,
            dispatch_uncaught_exception_called: false,
        }
    }

    pub fn run(&mut self) {
        let vm = vm::VM::new(3);

        //attach 'main' thread
        vm.threads.attach_current_thread();

        info!("init vm start");
        init_vm::initialize_jvm();
        info!("init vm end");

        let main_class = oop::class::load_and_init(self.class.as_bytes());

        let mir = {
            let cls = main_class.get_class();

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

            cls.get_static_method(&new_br("main"), &new_br("([Ljava/lang/String;)V"))
        };

        let jt = runtime::thread::current_java_thread();
        match mir {
            Ok(mir) => {
                let args = self.build_main_arg();
                let mut jc = JavaCall::new_with_args(mir, args);
                jt.write().unwrap().is_alive = true;
                jc.invoke(None, true);
                jt.write().unwrap().is_alive = false;
            }
            _ => unreachable!("NotFound \"main\""),
        }

        if jt.read().unwrap().ex.is_some() {
            self.uncaught_ex(main_class);
        }

        //detach main thread
        vm.threads.detach_current_thread();

        vm.threads.join_all();
    }
}

impl MainThread {
    fn build_main_arg(&self) -> Vec<Oop> {
        let args = match &self.args {
            Some(args) => args
                .iter()
                .map(|it| util::oop::new_java_lang_string2(it))
                .collect(),
            None => vec![],
        };

        //build ArrayOopDesc
        let ary_str_class = runtime::require_class3(None, b"[Ljava/lang/String;").unwrap();
        vec![Oop::new_ref_ary2(ary_str_class, args)]
    }

    fn uncaught_ex(&mut self, main_cls: ClassRef) {
        if self.dispatch_uncaught_exception_called {
            self.uncaught_ex_internal();
        } else {
            self.dispatch_uncaught_exception_called = true;
            self.call_dispatch_uncaught_exception(main_cls);
        }
    }

    fn call_dispatch_uncaught_exception(&mut self, main_cls: ClassRef) {
        let jt = runtime::thread::current_java_thread();
        let v = {
            let jt = jt.read().unwrap();
            jt.java_thread_obj.clone()
        };
        match v {
            Some(v) => {
                let cls = {
                    let rf = v.extract_ref();
                    let inst = rf.extract_inst();
                    inst.class.clone()
                };

                let mir = {
                    let cls = cls.get_class();
                    cls.get_this_class_method(
                        &new_br("dispatchUncaughtException"),
                        &new_br("(Ljava/lang/Throwable;)V"),
                    )
                };

                match mir {
                    Ok(mir) => {
                        let ex = {
                            let mut jt = jt.write().unwrap();
                            jt.take_ex().unwrap()
                        };
                        let args = vec![v, ex];
                        let mut jc = JavaCall::new_with_args(mir, args);
                        jc.invoke(None, false);
                    }
                    _ => self.uncaught_ex_internal(),
                }
            }

            None => self.uncaught_ex_internal(),
        }
    }

    fn uncaught_ex_internal(&mut self) {
        let jt = runtime::thread::current_java_thread();
        let ex = {
            let mut jt = jt.write().unwrap();
            jt.take_ex().unwrap()
        };

        let cls = {
            let rf = ex.extract_ref();
            let inst = rf.extract_inst();
            inst.class.clone()
        };

        let detail_message = {
            let fid = {
                let cls = cls.get_class();
                cls.get_field_id(&new_br("detailMessage"), &new_br("Ljava/lang/String;"), false)
            };
            let v = Class::get_field_value(ex.extract_ref(), fid);
            OopRef::java_lang_string(v.extract_ref())
        };
        let name = {
            let cls = cls.get_class();
            cls.name.clone()
        };

        let cls_name = String::from_utf8_lossy(name.as_slice());
        error!("Name={}, detailMessage={}", cls_name, detail_message);
    }
}
