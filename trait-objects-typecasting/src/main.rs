trait Animal{
    fn speak(&self);
}

trait Pet:Animal{
    fn name(&self) -> &str;
}

struct Dog{
    name: String,
}

impl Pet for Dog {
    fn name(&self) -> &str {
        self.name.as_str()
    }
}
impl Animal for Dog{
    fn speak(&self) {
        println!("{} says woff!", self.name());
    }
}

fn make_speak(animals:Vec<Box<dyn Animal>>) {
    for animal in animals {
        animal.speak();
    }
}

fn main() {
    let pets: Vec<Box<dyn Pet>> = vec![
        Box::new(Dog{name: "Mickey".into()}),
        Box::new(Dog{name: "Joey".into()}),
    ];
    let animals: Vec<Box<dyn Animal>> = pets.into_iter().map(|p| p as Box<dyn Animal>).collect();
    make_speak(animals);
}