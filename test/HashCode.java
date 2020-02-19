class HashCode 
{
    public static void main(String[] args)
    {
        String s1 = new String("abcde");
        String s2 = new String("abcde");
        String s3 = "abcde";
        System.out.println("s1 = " + s1.hashCode());
        System.out.println("s2 = " + s2.hashCode());
        System.out.println("s3 = " + s3.hashCode());
    }
} 
