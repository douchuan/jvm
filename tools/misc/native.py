#!/usr/bin/env python3

# -*- coding: utf-8 -*-
import sys


if __name__ == '__main__':
	desc = sys.argv[1]
	[package, name, signature] = desc.split(":")

	mod = package.replace("/", "_")
	print(mod)
	print()

	print("mod " + mod + ";")
	print()

	print("(\"" + package + "\", " + mod + "::get_native_methods()),")
	print()

	print("#![allow(non_snake_case)]")
	print()
	print("use crate::native::{new_fn, JNIEnv, JNINativeMethod, JNIResult};")
	print("use crate::oop::{self, Oop};")
	print()
	print("pub fn get_native_methods() -> Vec<JNINativeMethod> {")
	print("\tvec![")
	print()
	print("\t]")
	print("}")
	print()

	print("new_fn(\"" + name + "\", " + "\"" + signature + "\", " + "Box::new(jvm_" + name + ")),")
	print()

	print("fn jvm_" + name + "(_env: JNIEnv, _args: &Vec<Oop>) -> JNIResult {")
	print("\tOk(None)")
	print("}")
