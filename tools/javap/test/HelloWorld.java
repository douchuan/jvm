public final class HelloWorld
{
    int count;
    String name;

    public HelloWorld() {

    }

    public static void main(String[] args)
    {
        System.out.println("Hello, World!");

        System.out.println("args: " + args);
        for (String s: args) {
            System.out.println("arg: " + s);
        }

        if (args != null) {
            for (int i = 0; i < args.length; i++) {
                System.out.println("arg[" + i + "] = " + args[i]);
            }
        }
    }
} 
