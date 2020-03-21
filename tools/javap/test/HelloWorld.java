public final class HelloWorld
{
    public static void main(String[] args)
    {
        System.out.println("Hello, World!");

        System.out.println("args: " + args);
        for (String s: args) {
            System.out.println("arg: " + s);
        }
    }
} 
