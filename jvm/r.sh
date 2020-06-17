#########################################
#modify to according to your env
#
#If you installed JDK, but can not find jre path, try:
#java -XshowSettings:properties
#
#On Linux, maybe
#JAVA_HOME="/usr/lib/jvm/java-1.8.0-openjdk-amd64/jre"
#########################################
JAVA_HOME=/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre
########################################

MY_SAMPLE=sample

##############################################
function join_by { local IFS="$1"; shift; echo "$*"; }
##############################################

#FIXME: win should be ';'
SEP=':'

jars=$JAVA_HOME/lib/*.jar
JDK=$(join_by $SEP ${jars[@]})

export JAVA_HOME
cargo run -- --cp $JDK:$MY_SAMPLE HelloWorld
