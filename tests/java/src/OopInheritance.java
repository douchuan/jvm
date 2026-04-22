class Animal {
    protected String name;

    public Animal(String name) {
        this.name = name;
    }

    public String getName() {
        return name;
    }

    public String speak() {
        return "Some sound";
    }
}

class Dog extends Animal {
    private String breed;

    public Dog(String name, String breed) {
        super(name);
        this.breed = breed;
    }

    @Override
    public String speak() {
        return "Woof!";
    }

    public String getBreed() {
        return breed;
    }
}

class Cat extends Animal {
    public Cat(String name) {
        super(name);
    }

    @Override
    public String speak() {
        return "Meow!";
    }
}

public class OopInheritance {
    public static void main(String[] args) {
        Animal animal = new Animal("Generic");
        System.out.println(animal.getName() + " says " + animal.speak());

        Dog dog = new Dog("Buddy", "Labrador");
        System.out.println(dog.getName() + " says " + dog.speak());
        System.out.println(dog.getName() + " breed: " + dog.getBreed());

        Cat cat = new Cat("Whiskers");
        System.out.println(cat.getName() + " says " + cat.speak());
    }
}
