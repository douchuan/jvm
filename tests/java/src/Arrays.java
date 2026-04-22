public class Arrays {
    public static void main(String[] args) {
        // int array
        int[] nums = new int[5];
        for (int i = 0; i < nums.length; i++) {
            nums[i] = i * 10;
        }
        System.out.println("nums[3] = " + nums[3]);

        // long array
        long[] longs = new long[3];
        longs[0] = 100L;
        longs[1] = 200L;
        longs[2] = 300L;
        System.out.println("longs[2] = " + longs[2]);

        // String array
        String[] fruits = new String[]{"apple", "banana", "cherry"};
        System.out.println("fruits[1] = " + fruits[1]);

        // 2D array
        int[][] matrix = new int[3][3];
        for (int i = 0; i < 3; i++) {
            for (int j = 0; j < 3; j++) {
                matrix[i][j] = i * 3 + j;
            }
        }
        System.out.println("matrix[2][1] = " + matrix[2][1]);

        // array bounds check (should throw if index out of bounds)
        try {
            int[] small = new int[2];
            small[5] = 42;
            System.out.println("ERROR: should have thrown");
        } catch (ArrayIndexOutOfBoundsException e) {
            System.out.println("ArrayIndexOutOfBoundsException caught");
        }
    }
}
