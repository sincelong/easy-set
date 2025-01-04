use configuration::{Configuration, config_to_toml, core, get_configuration};

mod configuration;

fn main() {
    const FILE_NAME: &str = "configuration.toml";
    println!("Hello, world!");
    let mut configs = get_configuration(FILE_NAME);
    match configs {
        Ok(mut configs) => {
            // println!("{}", config_to_toml(&configs));
            core(&mut configs);
        }
        Err(_e) => {
            println!("{FILE_NAME} is no exist or Parse Error");
            // create_configuration(FILE_NAME).expect(&format!("Create File {FILE_NAME} Failed"));
            let mut configs: Vec<Configuration> = Vec::new();
            core(&mut configs);
        }
    }
}
