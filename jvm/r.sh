#########################################
###modify to according to your env#######
JAVA_HOME=/Library/Java/JavaVirtualMachines/jdk1.8.0_151.jdk/Contents/Home/jre
MY_SAMPLE=sample
########################################

##############################################
function join_by { local IFS="$1"; shift; echo "$*"; }
##############################################

#FIXME: win should be ';'
SEP=':'

jars=$JAVA_HOME/lib/*.jar
JDK=$(join_by $SEP ${jars[@]})

export JAVA_HOME
cargo run -- --cp $JDK:$MY_SAMPLE HelloWorld
