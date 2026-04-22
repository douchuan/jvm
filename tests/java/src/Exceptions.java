public class Exceptions {
    public static void main(String[] args) {
        // ArithmeticException
        try {
            int x = 1 / 0;
            System.out.println("ERROR: should have thrown");
        } catch (ArithmeticException e) {
            System.out.println("ArithmeticException caught: divide by zero");
        }

        // NullPointerException
        try {
            String s = null;
            s.length();
            System.out.println("ERROR: should have thrown");
        } catch (NullPointerException e) {
            System.out.println("NullPointerException caught");
        }

        // ArrayIndexOutOfBoundsException
        try {
            int[] a = new int[1];
            a[10] = 5;
            System.out.println("ERROR: should have thrown");
        } catch (ArrayIndexOutOfBoundsException e) {
            System.out.println("ArrayIndexOutOfBoundsException caught");
        }

        // Multiple catch blocks
        try {
            methodThatThrows();
        } catch (IllegalArgumentException e) {
            System.out.println("Caught IllegalArgumentException");
        }

        // finally block
        try {
            System.out.println("in try");
        } finally {
            System.out.println("finally executed");
        }

        // Nested try-catch
        try {
            try {
                throw new RuntimeException("inner");
            } catch (RuntimeException e) {
                System.out.println("Caught inner: " + e.getMessage());
                throw e;
            }
        } catch (RuntimeException e) {
            System.out.println("Caught outer: " + e.getMessage());
        }
    }

    static void methodThatThrows() {
        throw new IllegalArgumentException("test");
    }
}
