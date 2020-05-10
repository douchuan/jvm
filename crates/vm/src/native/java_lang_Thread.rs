#![allow(non_snake_case)]

use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};
use crate::oop::{self, Oop};
use crate::runtime::{self, vm, JavaCall, JavaThread};
use crate::types::JavaThreadRef;
use crate::util;
use crate::runtime::vm::get_vm;

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
    ]
}

fn jvm_registerNatives(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

fn jvm_currentThread(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    runtime::thread::THREAD.with(|jt| {
        let obj = jt.borrow().read().unwrap().java_thread_obj.clone();
        Ok(obj)
    })
}

fn jvm_setPriority0(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    Ok(None)
}

//'_jt' is caller's thread context, can't be used here
//should find by 'eetop' in thread pool
fn jvm_isAlive(_jt: JavaThreadRef, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let this = args.get(0).unwrap();
    let eetop = util::oop::extract_java_lang_thread_eetop(this);
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

fn jvm_start0(_jt: JavaThreadRef, _env: JNIEnv, args: Vec<Oop>) -> JNIResult {
    let thread_oop = args.get(0).unwrap().clone();
    let cls = {
        let thread_oop = util::oop::extract_ref(&thread_oop);
        let v = thread_oop.read().unwrap();
        match &v.v {
            oop::RefKind::Inst(inst) => inst.class.clone(),
            _ => unreachable!(),
        }
    };

    let name = {
        let cls = cls.read().unwrap();
        cls.name.clone()
    };

    if name.as_slice() == "java/lang/ref/Reference$ReferenceHandler".as_bytes() {
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

            let mir = {
                let cls = cls.read().unwrap();

                //setup eetop
                let eetop = jt.read().unwrap().eetop;
                let fid = cls.get_field_id(b"eetop", b"J", false);
                cls.put_field_value(thread_oop.clone(), fid, Oop::new_long(eetop));

                //obtain 'run' method
                cls.get_virtual_method(b"run", b"()V").unwrap()
            };

            //invoke 'run'
            let mut jc = JavaCall::new_with_args(mir, args);
            let area = runtime::DataArea::new(0, 0);
            jt.write().unwrap().is_alive = true;
            jt.write().unwrap().java_thread_obj = Some(thread_oop.clone());
            jc.invoke(jt.clone(), Some(area), false);
            jt.write().unwrap().is_alive = false;

            //notify thread that invoke 'join'
            let v = util::oop::extract_ref(&thread_oop);
            v.read().unwrap().notify_all();

            vm.threads.detach_current_thread();
        });

        Ok(None)
    }
}

fn jvm_isInterrupted(_jt: JavaThreadRef, _env: JNIEnv, _args: Vec<Oop>) -> JNIResult {
    //todo: fix me
    let v = Oop::new_int(0);
    Ok(Some(v))
}
