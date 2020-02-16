class Ex 
{
    public static void main(String[] args)
    {
        try {
            fn1();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    static void fn1() throws Exception {
        Exception ex = new Exception();
        throw ex;
    }
} 
