class SubOverflow 
{
    public static void main(String[] args)
    {

        //int overflow
        int i;

        i = Integer.MAX_VALUE + 1;
        System.out.println("overflow int, i = " + i);

        i = Integer.MAX_VALUE;
        i++;
        System.out.println("overflow int, i = " + i);

        i = Integer.MAX_VALUE;
        i += 1;
        System.out.println("overflow int, i = " + i);

        i = Integer.MAX_VALUE;
        ++i;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE - 1;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE;
        i--;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE;
        i -= 1;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE;
        --i;
        System.out.println("overflow int, i = " + i);

        i = Integer.MAX_VALUE * Integer.MAX_VALUE;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE * Integer.MIN_VALUE;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE / 2;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE % 2;
        System.out.println("overflow int, i = " + i);

        i = Integer.MIN_VALUE >> 1;
        System.out.println("overflow int, i = " + i);

        i = Integer.MAX_VALUE << 1;
        System.out.println("overflow int, i = " + i);

        i = (int)Long.MAX_VALUE;
        System.out.println("overflow int, i = " + i);

        i = (int)Long.MIN_VALUE;
        System.out.println("overflow int, i = " + i);

        i = 100;
        i <<= 1000;
        System.out.println("overflow int, i = " + i);

        i = 100;
        i >>= 1000;
        System.out.println("overflow int, i = " + i);


        ////////////////////////////////////////////////
        //long overflow
        long l;

        l = Long.MAX_VALUE + 1;
        System.out.println("overflow long, l = " + l);

        l = Long.MAX_VALUE;
        l++;
        System.out.println("overflow long, l = " + l);

        l = Long.MAX_VALUE;
        ++l;
        System.out.println("overflow long, l = " + l);

        l = Long.MIN_VALUE;
        l--;
        System.out.println("overflow long, l = " + l);

        l = Long.MIN_VALUE;
        --l;
        System.out.println("overflow long, l = " + l);

        l = Long.MIN_VALUE - 1;
        System.out.println("overflow long, l = " + l);

        l = -Long.MIN_VALUE - Long.MIN_VALUE;
        System.out.println("overflow long = " + l);

        l = Long.MAX_VALUE * Long.MAX_VALUE;
        System.out.println("overflow long = " + l);

        l = Long.MIN_VALUE * Long.MIN_VALUE;
        System.out.println("overflow long = " + l);

        l = Long.MIN_VALUE / 2;
        System.out.println("overflow long = " + l);

        l = Long.MIN_VALUE % 2;
        System.out.println("overflow long = " + l);

        l = Long.MIN_VALUE >> 1;
        System.out.println("overflow long = " + l);

        l = Long.MAX_VALUE << 1;
        System.out.println("overflow long = " + l);

        l = Long.MAX_VALUE << 1000;
        System.out.println("overflow long = " + l);

    }
} 
