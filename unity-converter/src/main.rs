use std::io;

fn main() {
    println!("==================== Unity Converter ====================");
    println!("1. Convert from Celsius to Fahrenheit");
    println!("2. Convert from Fahrenheit to Celsius");
    println!("3. Convert from Kilometers to Miles");
    println!("4. Convert from Miles to Kilometers");
    println!("5. Convert from Kilograms to Pounds");
    println!("6. Convert from Pounds to Kilograms");  
    println!("=========================================================");

    let mut line = String::new();

    println!("Choose an option:");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let option: u32 = match line.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("I only accept integers numbers !");
            return;
        }
    };

    line.clear();

    println!("What is the value you want to convert:");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let value: f64 = match line.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("I only accept numbers !");
            return;
        }
    };

    match option{
    1 => println!("Result: {:.2} °F", (value*9.0 / 5.0) + 32.0),
    2 => println!("Result: {:.2} °C", (value-32.0 )*5.0 / 9.0),
    3 => println!("Result: {:.2} Miles", value * 0.621371),
    4 => println!("Result: {:.2} Kilometers", value / 0.621371),
    5 => println!("Result: {:.2} Pounds", value * 2.20462),
    6 => println!("Result: {:.2} Kilograms", value / 2.20462),
    _=>println!("You funny guy !")
    }
}
