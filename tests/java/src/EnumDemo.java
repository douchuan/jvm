public enum EnumDemo {
    MONDAY(1),
    TUESDAY(2),
    WEDNESDAY(3),
    THURSDAY(4),
    FRIDAY(5),
    SATURDAY(6),
    SUNDAY(7);

    private final int dayNumber;

    EnumDemo(int dayNumber) {
        this.dayNumber = dayNumber;
    }

    public int getDayNumber() {
        return dayNumber;
    }

    public String getType() {
        if (this == SATURDAY || this == SUNDAY) {
            return "Weekend";
        }
        return "Weekday";
    }

    public static void main(String[] args) {
        EnumDemo today = EnumDemo.WEDNESDAY;
        System.out.println("Today: " + today);
        System.out.println("Day number: " + today.getDayNumber());
        System.out.println("Type: " + today.getType());

        // ordinal and name
        System.out.println("ordinal: " + today.ordinal());
        System.out.println("name: " + today.name());

        // values()
        EnumDemo[] all = EnumDemo.values();
        System.out.println("Total days: " + all.length);

        for (EnumDemo day : all) {
            System.out.println(day + " = " + day.getDayNumber() + " (" + day.getType() + ")");
        }
    }
}
