public final class HelloWorld
{
    float v_float = 2.5f;
    double v_double = 2.0;
    int v_int = 100;
    long v_long = 20000l;

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

    private void private_method() {
        System.out.println("I'm private method");
    }

    protected void protected_method() {
        System.out.println("I'm protected method");
    }

    void package_method() {
        System.out.println("I'm package method");
    }

    public void public_method() {
        System.out.println("I'm public method");
    }
} 
