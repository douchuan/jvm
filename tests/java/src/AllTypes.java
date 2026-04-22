public class AllTypes {
    public byte b;
    public short s;
    public int i;
    public long l;
    public float f;
    public double d;
    public char c;
    public boolean z;
    public String str;
    public Object obj;
    public int[] arr;
    public String[] strs;

    public static void main(String[] args) {
        AllTypes t = new AllTypes();
        t.b = 127;
        t.s = 32767;
        t.i = 2147483647;
        t.l = 9223372036854775807L;
        t.f = 3.14f;
        t.d = 2.718281828;
        t.c = 'A';
        t.z = true;
        t.str = "hello";
        t.obj = new Object();
        t.arr = new int[]{1, 2, 3};
        t.strs = new String[]{"one", "two"};
        System.out.println("AllTypes initialized OK");
        System.out.println("b=" + t.b + " s=" + t.s + " i=" + t.i);
        System.out.println("l=" + t.l + " f=" + t.f + " d=" + t.d);
        System.out.println("c=" + t.c + " z=" + t.z);
        System.out.println("str=" + t.str + " arr[0]=" + t.arr[0]);
    }
}
