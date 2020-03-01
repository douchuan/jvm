class Ex 
{
    public static void main(String[] args)
    {
        try {
            fn1();
        } catch (Exception e) {
            e.printStackTrace();
        }

        try {
            fn2();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    static void fn1() throws Exception {
        Exception ex = new Exception();
        throw ex;
    }

    static void fn2() {
        int sum = 0;
        int d = 0;
        for (int i = 0; i < 10; i ++) {
            sum += i;

            d = i - i;
        }

        int re = 1000 / d;

        System.out.println("sum = " + sum);
    }
} 
