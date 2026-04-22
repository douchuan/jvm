public class StaticInit {
    // Static fields initialized at class load time
    static int counter = 0;
    static String message;
    static final double PI = 3.141592653589793;

    // Static initialization block
    static {
        message = "Initialized in static block";
        counter = 100;
        System.out.println("Static block executed");
    }

    // Instance fields
    private int id;

    public StaticInit(int id) {
        this.id = id;
        counter++;
    }

    public int getId() {
        return id;
    }

    public static int getCounter() {
        return counter;
    }

    public static String getMessage() {
        return message;
    }

    public static void main(String[] args) {
        System.out.println("Message: " + StaticInit.getMessage());
        System.out.println("PI: " + PI);

        StaticInit a = new StaticInit(1);
        StaticInit b = new StaticInit(2);

        System.out.println("Counter after 2 instances: " + StaticInit.getCounter());
        System.out.println("A id: " + a.getId());
        System.out.println("B id: " + b.getId());
    }
}
