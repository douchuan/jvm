public class Strings {
    public static void main(String[] args) {
        // String creation
        String hello = "Hello";
        String world = "World";

        // Concatenation
        String greeting = hello + ", " + world + "!";
        System.out.println(greeting);

        // length
        System.out.println("Length of greeting: " + greeting.length());

        // charAt
        System.out.println("First char: " + greeting.charAt(0));
        System.out.println("Last char: " + greeting.charAt(greeting.length() - 1));

        // equals
        String s1 = new String("test");
        String s2 = new String("test");
        System.out.println("s1 equals s2: " + s1.equals(s2));
        System.out.println("s1 == s2: " + (s1 == s2));

        // substring
        String sub = greeting.substring(0, 5);
        System.out.println("Substring: " + sub);

        // StringBuilder (tests invokevirtual chain)
        StringBuilder sb = new StringBuilder();
        sb.append("one");
        sb.append("-");
        sb.append("two");
        System.out.println("StringBuilder: " + sb.toString());

        // String format with numbers
        int num = 42;
        System.out.println("The answer is: " + num);
    }
}
