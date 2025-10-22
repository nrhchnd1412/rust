struct Counter{
    count: u32,
    max: u32,
}

impl Counter{
    fn new(max:u32) -> Counter{
        Counter{
            count:0,
            max
        }
    }
}

impl Iterator for Counter{
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item>{
        self.count += 1;
        if self.count > self.max{
            None
        }
        else{
            self.count += 1;
            Some(self.count)
        }
    }
}

fn main() {
    let counter = Counter::new(10);
    for item in counter{
        println!("{}", item);
    }
}
