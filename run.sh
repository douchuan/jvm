
### not use zulu8, cause run $JDK_TEST/Character/CheckScript failed
#JDK_PATH=test/zulu8/jre/lib
#export JAVA_HOME=/Users/douchuan/work/prj_rust/jvm/test/zulu8/jre


JDK_PATH=/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre/lib
JDK=$JDK_PATH/resources.jar:$JDK_PATH/rt.jar:$JDK_PATH/jsse.jar:$JDK_PATH/jce.jar:$JDK_PATH/charsets.jar:$JDK_PATH/jfr.jar

JDK_TEST=/Users/douchuan/work/codes/java/openjdk-8u41-src-b04-14_jan_2020/openjdk/jdk/test/java/lang
MY_TEST=.:./test

export JAVA_HOME=/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre

#export RUST_LOG=trace
#export RUST_LOG=info
#export RUST_LOG=warn
export RUST_BACKTRACE=full



### My Test
#cargo run -- --cp $JDK:$MY_TEST Add
#cargo run -- --cp $JDK:$MY_TEST HelloWorld 123 456 789
#cargo run -- --cp $JDK:$MY_TEST HelloWorldUnicode
#cargo run -- --cp $JDK:$MY_TEST Ex
#cargo run -- --cp $JDK:$MY_TEST MyFile
#cargo run -- --cp $JDK:$MY_TEST MyInteger
#cargo run -- --cp $JDK:$MY_TEST MyArrayCopy

### fix Overflow
#cargo run -- --cp $JDK:$MY_TEST SubOverflow

### fix Enum CloneNotSupportedException
#cargo run -- --cp $JDK:$MY_TEST EnumDemo

### fix System.out.printf not work, resolve_again for acc_flags == 0
#cargo run -- --cp $JDK:$MY_TEST Printf

### fix ThreadLocal not work, resolve_again for protected
#cargo run -- --cp $JDK:$MY_TEST ThreadLocalTest

### load with custom package
## should panic
#cargo run -- --cp $JDK:$MY_TEST test/with_package/my.ns.HelloWorld
## ok
#cargo run -- --cp $JDK:$MY_TEST:test/with_package my.ns.HelloWorld


###regex
## System.out.printf not work
##
## 原因1: invoke_virtual定位method错误
##  因为java.util.regex.Pattern$Node.match方法的acc_flags==0,
##导致没有resolve_again，acc_flags为0的含义是什么?
##  java.util.regex.Pattern$Node.match (错误的找到这个method)
##  java.util.regex.Pattern$Start.match (应该用这个)
##
## 原因2：
##   java.util.Formatter$FormatSpecifier.conversion(String)
## 调用Character.toLowerCase(c)转换错误
##   码表在java/lang/CharacterDataLatin1中, ldc CONSTANT_String_info 加载
##
## Modified UTF-8 strings 编码定义:
## JVM Spec, 4.4.7 The CONSTANT_Utf8_info Structure 定义
#cargo run -- --cp $JDK:$MY_TEST:./test/regex Printf

###
##Float.toString(1.0f) crash
##ThreadLocal.initialValue not called, so NPE happend
#cargo run -- --cp $JDK:$MY_TEST:./test/float ToString
#cargo run -- --cp $JDK:$MY_TEST:./test/char MyCheckScript

###############################
### jdk test
###############################
#cargo run -- --cp $JDK:$JDK_TEST Compare
#cargo run -- --cp $JDK:$JDK_TEST HashCode
#cargo run -- --cp $JDK:$JDK_TEST ToString

###todo: optimize
###init vm，初始化安全模块慢。
### File.createTempFile，会使用SecureRandom，导致一系列安全相关的类被加载
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Appendable Basic
#cargo run --release -- --cp $JDK:$JDK_TEST:$JDK_TEST/Appendable Basic

#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/AssertionError Cause
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Boolean Factory
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Boolean GetBoolean
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Boolean MakeBooleanComparable
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Boolean ParseBoolean
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Byte Decode
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Class/asSubclass BasicUnit
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Math AbsPositiveZero

##todo: depend on testng
##IntegralPrimitiveToString.java, PrimitiveSumMinMaxTest.java
##CharSequence/DefaultTest.java

##todo: optimize
################################
### oracle java
#sum_t_list_add = 16
#sum_t_map_get = 1
#sum_t_map_put = 1
#sum_t_parse_int = 3
#sum_t_println = 26
#sum_t_int2integer = 13
#################################
### mine debug mode
#sum_t_list_add = 45778
#sum_t_map_get = 867
#sum_t_map_put = 63
#sum_t_parse_int = 4502
#sum_t_println = 21181
#sum_t_int2integer = 27745
### mine release mode
#sum_t_list_add = 5494
#sum_t_map_get = 123
#sum_t_map_put = 8
#sum_t_parse_int = 626
#sum_t_println = 3059
#sum_t_int2integer = 3201
export TEST_SRC=/Users/douchuan/work/codes/java/openjdk-8u41-src-b04-14_jan_2020/openjdk/jdk/test/java/lang/Character
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Character MyCheckProp
#cargo run --release -- --cp $JDK:$JDK_TEST:$JDK_TEST/Character MyCheckProp
#cargo run --release -- --cp $JDK:$JDK_TEST:$JDK_TEST/Character CheckProp
#cargo run --release -- --cp $JDK:$JDK_TEST:$JDK_TEST/Character CheckScript
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Class ArrayMethods


