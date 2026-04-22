public class Arithmetic {
    public static void main(String[] args) {
        // int arithmetic
        int a = 2147483640;
        int b = 10;
        System.out.println("int add: " + (a + b));
        System.out.println("int sub: " + (a - b));
        System.out.println("int mul: " + (a * 3));

        // long arithmetic
        long la = 9223372036854775800L;
        long lb = 100L;
        System.out.println("long add: " + (la + lb));
        System.out.println("long mul: " + (la * 2L));

        // float arithmetic
        float fa = 3.14f;
        float fb = 2.0f;
        System.out.println("float div: " + (fa / fb));

        // double arithmetic
        double da = 1.41421356237;
        double db = 2.0;
        System.out.println("double mul: " + (da * db));

        // type conversion
        int i = 100;
        long l = i;
        float f = i;
        double d = i;
        System.out.println("int to long: " + l);
        System.out.println("int to float: " + f);
        System.out.println("int to double: " + d);

        // bitwise ops
        int x = 0b1100;
        int y = 0b1010;
        System.out.println("and: " + (x & y));
        System.out.println("or: " + (x | y));
        System.out.println("xor: " + (x ^ y));
        System.out.println("shl: " + (x << 2));
        System.out.println("shr: " + (x >> 1));
        System.out.println("ushr: " + (-1 >>> 1));
    }
}
