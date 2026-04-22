#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::new_br;
use crate::oop::{self, Class, Oop};
use crate::runtime::vm::get_vm;
use crate::runtime::{self, vm, JavaCall, JavaThread};
use tracing::{debug, error, info, trace, warn};

pub fn get_native_methods() -> Vec<JNINativeMethod> {
    vec![
        new_fn("registerNatives", "()V", Box::new(jvm_registerNatives)),
        new_fn(
            "currentThread",
            "()Ljava/lang/Thread;",
            Box::new(jvm_currentThread),
        ),
        new_fn("setPriority0", "(I)V", Box::new(jvm_setPriority0)),
        new_fn("isAlive", "()Z", Box::new(jvm_isAlive)),
        new_fn("start0", "()V", Box::new(jvm_start0)),
        new_fn("isInterrupted", "(Z)Z", Box::new(jvm_isInterrupted)),
        // JDK 9+: dispatchUncaughtException calls threadState/isTerminated which aren't implemented.
        // Implement as native to print exception and skip the cascade.
        new_fn(
            "dispatchUncaughtException",
            "(Ljava/lang/Throwable;)V",
            Box::new(jvm_dispatchUncaughtException),
        ),
    ]
}

fn jvm_registerNatives(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

fn jvm_currentThread(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    let jt = runtime::thread::current_java_thread();
    let obj = jt.read().unwrap().java_thread_obj.clone();
    Ok(obj)
}

fn jvm_setPriority0(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    Ok(None)
}

//'_jt' is caller's thread context, can't be used here
//should find by 'eetop' in thread pool
fn jvm_isAlive(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let this = args.get(0).unwrap();
    let eetop = Oop::java_lang_thread_eetop(this.extract_ref());
    let vm = get_vm();

    let r = match vm.threads.find_java_thread(eetop) {
        Some(jt) => {
            info!("native thread tag = {}", jt.read().unwrap().tag);
            if jt.read().unwrap().is_alive {
                1
            } else {
                0
            }
        }
        None => 0,
    };

    Ok(Some(Oop::new_int(r)))
}

fn jvm_start0(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let thread_oop = args.get(0).unwrap().clone();
    let clazz = oop::with_heap(|heap| {
        let slot_id = thread_oop.extract_ref();
        let desc = heap.get(slot_id);
        let guard = desc.read().unwrap();
        guard.v.extract_inst().class.clone()
    });

    let cls = clazz.get_class();
    if cls.name.as_slice() == "java/lang/ref/Reference$ReferenceHandler".as_bytes() {
        Ok(None)
    } else {
        let vm = vm::get_vm();

        let jt = JavaThread::new(None, vm.threads.next_id());
        vm.threads.attach_java_thread(jt.clone());

        let args = vec![thread_oop.clone()];
        vm.threads.spawn_java_thread(move || {
            //setup current thread
            let current_thread = jt.clone();
            runtime::thread::THREAD.with(|t| {
                *t.borrow_mut() = current_thread;
            });

            let cls = clazz.get_class();
            let mir = {
                //setup eetop
                let eetop = jt.read().unwrap().eetop;
                let fid = cls.get_field_id(&new_br("eetop"), &new_br("J"), false);
                Class::put_field_value2(thread_oop.extract_ref(), fid.offset, Oop::new_long(eetop));

                //obtain 'run' method
                cls.get_virtual_method(&new_br("run"), &new_br("()V"))
                    .unwrap()
            };

            //invoke 'run'
            let mut jc = JavaCall::new_with_args(mir, args);
            jt.write().unwrap().is_alive = true;
            jt.write().unwrap().java_thread_obj = Some(thread_oop.clone());
            jc.invoke(None, false);
            jt.write().unwrap().is_alive = false;

            //notify thread that invoke 'join'
            let v = thread_oop.extract_ref();
            Oop::Ref(v).notify_all();

            vm.threads.detach_current_thread();
        });

        Ok(None)
    }
}

fn jvm_isInterrupted(_env: JNIEnv, _args: &[Oop]) -> JNIResult {
    //todo: fix me
    let v = Oop::new_int(0);
    Ok(Some(v))
}

fn jvm_dispatchUncaughtException(_env: JNIEnv, args: &[Oop]) -> JNIResult {
    let ex = args.get(1).unwrap();
    let cls = {
        let rf = ex.extract_ref();
        oop::with_heap(|heap| {
            let desc = heap.get(rf);
            let guard = desc.read().unwrap();
            guard.v.extract_inst().class.clone()
        })
    };
    let cls_name = String::from_utf8_lossy(cls.get_class().name.as_slice());
    error!("Uncaught exception: {}", cls_name);
    Ok(None)
}
