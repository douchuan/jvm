public class ControlFlow {
    public static void main(String[] args) {
        // if-else
        int x = 10;
        if (x > 5) {
            System.out.println("x > 5");
        } else {
            System.out.println("x <= 5");
        }

        // for loop
        int sum = 0;
        for (int i = 0; i < 10; i++) {
            sum += i;
        }
        System.out.println("sum 0..9 = " + sum);

        // while loop
        int count = 0;
        int n = 100;
        while (n > 1) {
            n /= 2;
            count++;
        }
        System.out.println("log2(100) = " + count);

        // do-while
        int total = 0;
        int j = 1;
        do {
            total += j;
            j++;
        } while (j <= 5);
        System.out.println("sum 1..5 = " + total);

        // switch
        int day = 3;
        String name;
        switch (day) {
            case 1: name = "Mon"; break;
            case 2: name = "Tue"; break;
            case 3: name = "Wed"; break;
            case 4: name = "Thu"; break;
            case 5: name = "Fri"; break;
            default: name = "Weekend"; break;
        }
        System.out.println("day 3 = " + name);

        // break / continue
        for (int i = 0; i < 20; i++) {
            if (i % 2 == 0) continue;
            if (i > 10) break;
            System.out.println("odd: " + i);
        }
    }
}
