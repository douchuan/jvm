public class SimpleCalc {
    public int add(int a, int b) { return a + b; }
    public long multiply(long a, long b) { return a * b; }
    public float divide(float a, float b) { return a / b; }
    public double subtract(double a, double b) { return a - b; }

    public static void main(String[] args) {
        SimpleCalc calc = new SimpleCalc();
        System.out.println("10 + 20 = " + calc.add(10, 20));
        System.out.println("6 * 7 = " + calc.multiply(6, 7));
        System.out.println("10.0 / 3.0 = " + calc.divide(10.0f, 3.0f));
        System.out.println("100.0 - 42.0 = " + calc.subtract(100.0, 42.0));
    }
}
