use zmq;

struct Person {
    age: i32,
}

impl Person {
    fn new(age: i32) -> Self {
        Person { age }
    }

    fn get_age(&self) -> i32 {
        self.age
    }
}

fn main() {
    // Print program description
    println!("This tool generates a CurveZMQ keypair, as two printable strings you can");
    println!("use in configuration files or source code. The encoding uses Z85, which");
    println!("is a base-85 format that is described in 0MQ RFC 32, and which has an");
    println!("implementation in the z85_codec.h source used by this tool. The keypair");
    println!("always works with the secret key held by one party and the public key");
    println!("distributed (securely!) to peers wishing to connect to it.");

    // Generate curve keypair
    match zmq::curve_keypair() {
        Ok((public_key, secret_key)) => {
            println!("\n== CURVE PUBLIC KEY ==");
            println!("{}", public_key);
            println!("\n== CURVE SECRET KEY ==");
            println!("{}", secret_key);
        }
        Err(e) => {
            if e.to_string().contains("not supported") {
                println!("To use curve_keygen, please install libsodium and then rebuild libzmq.");
            }
            std::process::exit(1);
        }
    }

    // Demonstrate Person functionality
    let person = Person::new(25);
    println!("\nPerson's age: {}", person.get_age());

    std::process::exit(0);
}