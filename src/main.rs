mod alu;
mod memory;
mod control;
use std::io;

fn main() {
    println!("STACK85 Test Driver, Ctrl+C to exit");
    println!("Available functions are:");
    println!("1> Test ALU");
    println!("2> Test Memory");
    println!("3> Test Program");
    loop {
        println!("Enter your choice:");
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        choice = choice.trim().to_string();

        if choice == "1" {
            test_alu();
            break;
        } else if choice == "2" {
            test_memory();
            break;
        } else if choice == "3" {
            control::test_pgm();
            break;
        } else {
            println!("Please enter a valid option.");
            continue;
        }
    }
}

fn test_alu() {
    let mut alu = alu::new();

    loop {
        println!("Enter X:");
        let mut x = String::new();
        io::stdin().read_line(&mut x).expect("Failed to read line");

        let x: u8 = match x.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("X: Please enter a number from 0 to 255.");
                continue;
            }
        };

        println!("Enter Y:");
        let mut y = String::new();
        io::stdin().read_line(&mut y).expect("Failed to read line");

        let y: u8 = match y.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Y: Please enter a number from 0 to 255.");
                continue;
            }
        };

        println!("Enter opcode:");
        let mut op = String::new();
        io::stdin().read_line(&mut op).expect("Failed to read line");

        let op: u8 = match op.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!(
                    "Opcode: Please enter a number from 0 to {}.",
                    alu::ALU_MAX_OPCODE
                );
                continue;
            }
        };

        if op > alu::ALU_MAX_OPCODE {
            println!(
                "Opcode: Please enter a number from 0 to {}.",
                alu::ALU_MAX_OPCODE
            );
            continue;
        }

        alu.load_x(x as u8);
        alu.load_y(y as u8);
        alu.load_op(op);
        alu.compute();

        println!(
            "Result: {} ({}), Mult: {}, ONZVC: {:05b}",
            alu.result(),
            alu.result() as i8,
            alu.result() as u16 | (alu.res_hi() as u16) << 8,
            alu.flags()
        );
    }
}

fn test_memory() {
    let mut memory = memory::new(memory::MEM_SIZE);

    loop {
        println!("(R)ead or (W)rite:");
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read line");

        if !choice.trim().eq_ignore_ascii_case("r") && !choice.trim().eq_ignore_ascii_case("w") {
            println!("Please enter R or W.");
            continue;
        }

        println!("Address:");
        let mut address = String::new();
        io::stdin()
            .read_line(&mut address)
            .expect("Failed to read line");

        let address: u16 = match address.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!(
                    "Address: Please enter a number from 0 to {}.",
                    memory::MEM_SIZE - 1
                );
                continue;
            }
        };

        if address >= memory::MEM_SIZE {
            println!(
                "Address: Please enter a number from 0 to {}.",
                memory::MEM_SIZE - 1
            );
            continue;
        }

        memory.set_addr(address);

        if choice.trim().eq_ignore_ascii_case("w") {
            println!("Enter value:");
            let mut data = String::new();
            io::stdin().read_line(&mut data).expect("Failed to read line");
            let data: u8 = match data.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Value: Please enter a number from 0 to 255.");
                    continue;
                }
            };

            memory.write(data);
            println!("Wrote {} to address {}.", data, address);
        } else {
            println!("Read {}.", memory.read());
        }
    }
}
