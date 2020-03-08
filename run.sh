
#########################################
###modify to according to your env
JAVA_HOME=/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre
JDK_SRC=/Users/douchuan/work/codes/java/openjdk-8u41-src-b04-14_jan_2020/openjdk
########################################

JDK=$JAVA_HOME/lib/resources.jar:$JAVA_HOME/lib/rt.jar:$JAVA_HOME/lib/jsse.jar:$JAVA_HOME/lib/jce.jar:$JAVA_HOME/lib/charsets.jar:$JAVA_HOME/lib/jfr.jar
JDK_T_LANG=$JDK_SRC/jdk/test/java/lang
MY_TEST=.:./test

export JAVA_HOME

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
#cargo run -- --cp $JDK:$JDK_T_LANG Compare
#cargo run -- --cp $JDK:$JDK_T_LANG HashCode
#cargo run -- --cp $JDK:$JDK_T_LANG ToString

###todo: optimize
###init vm，初始化安全模块慢。
### File.createTempFile，会使用SecureRandom，导致一系列安全相关的类被加载
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Appendable Basic
#cargo run --release -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Appendable Basic

#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/AssertionError Cause
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Boolean Factory
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Boolean GetBoolean
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Boolean MakeBooleanComparable
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Boolean ParseBoolean
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Byte Decode
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class/asSubclass BasicUnit
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Math AbsPositiveZero

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
export TEST_SRC=$JDK_SRC/jdk/test/java/lang/Character
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Character MyCheckProp
#cargo run --release -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Character MyCheckProp
#cargo run --release -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Character CheckProp
#cargo run --release -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Character CheckScript
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class ArrayMethods
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class Cast
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class IsAnnotationType
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class IsEnum
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class/forName InitArg
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class/forName InvalidNameWithSlash



#[2020-03-07T23:14:59Z ERROR jvm::native::sun_reflect_ConstantPool] s = LExpectedGenericString;
#[2020-03-07T23:14:59Z ERROR jvm::runtime::frame] mirror MyGenericStringTest cannot be cast to java.lang.reflect.Constructor
#Exception in thread "main" java.lang.ClassCastException: mirror MyGenericStringTest cannot be cast to java.lang.reflect.Constructor
#	at sun.reflect.generics.factory.CoreReflectionFactory.getDeclsLoader(CoreReflectionFactory.java:67)
#	at sun.reflect.generics.factory.CoreReflectionFactory.makeNamedType(CoreReflectionFactory.java:115)
#	at sun.reflect.generics.visitor.Reifier.visitClassTypeSignature(Reifier.java:125)
#	at sun.reflect.generics.tree.ClassTypeSignature.accept(ClassTypeSignature.java:49)
#	at sun.reflect.annotation.AnnotationParser.parseSig(AnnotationParser.java:439)
#	at sun.reflect.annotation.AnnotationParser.parseAnnotation2(AnnotationParser.java:241)
#	at sun.reflect.annotation.AnnotationParser.parseAnnotations2(AnnotationParser.java:120)
#	at sun.reflect.annotation.AnnotationParser.parseAnnotations(AnnotationParser.java:72)
#	at java.lang.Class.createAnnotationData(Class.java:3521)
#	at java.lang.Class.annotationData(Class.java:3510)
#	at java.lang.Class.getAnnotation(Class.java:3415)
#	at MyGenericStringTest.main(MyGenericStringTest.java:53)
###
###  the impl of getRawAnnotations & intance_of & classloader is not right
#cargo run -- --cp $JDK:$JDK_T_LANG:$JDK_T_LANG/Class MyGenericStringTest
