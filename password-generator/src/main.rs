use std::io;
use rand::Rng;

fn main() {
    println!("==================== Password Generator ====================");
    println!("You can choose the length and complexity of your password and the number of passwords you want to generate");
    println!("============================================================");

    let mut line = String::new();
    let mut rng = rand::rng();
    let mut chars = String::from("abcdefghijklmnopqrstuvwxyz");

    println!("How many password do you want ?");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let nb_password: u8 = match line.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("I only accept integers numbers !");
            return;
        }
    };

    line.clear();

    println!("What length ?");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let length: u8 = match line.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("I only accept integers numbers !");
            return;
        }
    };


    line.clear();

    println!("Do you want special charater (Y/n) ?");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let response = line.trim().to_lowercase();
    if response.is_empty() || response == "y" || response == "yes" {
        chars.push_str("!@#$%^&*()_+-=[]{}|;:,.<>?/`~");
    }

    line.clear();

    println!("Do you want numbers (Y/n) ?");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let response = line.trim().to_lowercase();
    if response.is_empty() || response == "y" || response == "yes" {
        chars.push_str("0123456789");
    }

    line.clear();

    println!("Do you want capital letters (Y/n) ?");
    io::stdin().read_line(&mut line).expect("Failed to read input");

    let response = line.trim().to_lowercase();
    if response.is_empty() || response == "y" || response == "yes" {
        chars.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    println!("============================================================");

    for _ in 0..nb_password {
        let password: String = (0..length)
            .map(|_| {
                let index = rng.random_range(0..chars.len());
                chars.chars().nth(index).unwrap()
            })
            .collect();

        println!("{}", password);
    }

    println!("============================================================");
}