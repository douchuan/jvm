JDK=test/zulu8/jre/lib/rt.jar:test/zulu8/jre/lib/charsets.jar

JDK_TEST=/Users/douchuan/work/codes/java/openjdk-8u41-src-b04-14_jan_2020/openjdk/jdk/test/java/lang
MY_TEST=.:./test

#export RUST_LOG=trace
#export RUST_LOG=info
#export RUST_BACKTRACE=1

### My Test
#cargo run -- --cp $JDK:$MY_TEST test/Add
#cargo run -- --cp $JDK:$MY_TEST test/HelloWorld 123 456 789
#cargo run -- --cp $JDK:$MY_TEST test/Ex

### fix Overflow
#cargo run -- --cp $JDK:$MY_TEST test/SubOverflow

### fix Enum CloneNotSupportedException
#cargo run -- --cp $JDK:$MY_TEST test/EnumDemo

### fix System.out.printf not work, resolve_again for acc_flags == 0
#cargo run -- --cp $JDK:$MY_TEST test/Printf

### fix ThreadLocal not work, resolve_again for protected
#cargo run -- --cp $JDK:$MY_TEST test/ThreadLocalTest


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
#cargo run -- --cp $JDK:$MY_TEST:./test/regex test/regex/Printf

###
##Float.toString(1.0f) crash
##ThreadLocal.initialValue not called, so NPE happend
#cargo run -- --cp $JDK:$MY_TEST:./test/float test/float/ToString

###############################
### jdk test
###############################

#cargo run -- --cp $JDK:$JDK_TEST Compare
#cargo run -- --cp $JDK:$JDK_TEST HashCode
#cargo run -- --cp $JDK:$JDK_TEST ToString
cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Appendable Appendable/Basic
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Class/asSubclass Class/asSubclass/BasicUnit
#cargo run -- --cp $JDK:$JDK_TEST:$JDK_TEST/Math Math/AbsPositiveZero

##todo: depend on testng
##IntegralPrimitiveToString.java, PrimitiveSumMinMaxTest.java

