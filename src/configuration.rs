use gethostname::gethostname;
use prettytable::row;
use serde::de::Error;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use toml::Table;
use winreg::RegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_WRITE};

#[derive(Debug, Default, Clone)]
pub struct JavaConfiguration {
    pub name: String,
    pub path: String,
}

pub struct TomlConfiguration {
    back_path: String,
    java_configuration: Vec<JavaConfiguration>,
}

#[derive(Debug, Default)]
pub struct Configuration {
    pub host_name: String,
    pub back_path: String,
    pub java_configuration: Vec<JavaConfiguration>,
}

impl Configuration {
    fn show_back(&self) {
        println!("当前配置环境变量备份为:");
        println!("{}", self.back_path);
    }

    fn set_back(&mut self) {
        self.show_back();

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let path: String = hklm
            .open_subkey("SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment")
            .unwrap()
            .get_value("Path")
            .unwrap();

        println!("当前系统环境变量备份为：");
        println!("{}", &path);

        println!("是否备份当前环境变量: (Y:default/N)");
        let mut input_string = String::new();

        io::stdin()
            .read_line(&mut input_string)
            .expect("无法读取输入");

        if input_string.trim() == "N" {
            return;
        } else {
            self.back_path = path;
        }
        println!("当前环境变量备份成功");
    }
}

#[derive(PartialEq, Debug)]
enum Command {
    Add,
    Change,
    Del,
    Exit,
    Show,
}

#[derive(Debug)]
struct MenuCommand {
    command: Command,
    path_id: u32,
}

impl MenuCommand {
    fn new(s: &str) -> Result<MenuCommand, String> {
        let mut menu_command: MenuCommand = MenuCommand {
            command: Command::Exit,
            path_id: 0,
        };

        let mut parts = s.trim().split_whitespace();
        let command = match parts.next().ok_or("命令获取失败")? {
            "A" => Command::Add,
            "C" => Command::Change,
            "D" => Command::Del,
            "E" => Command::Exit,
            "S" => Command::Show,
            _ => return Err("命令解析失败".to_string()),
        };

        menu_command.command = command;

        if matches!(
            menu_command.command,
            Command::Exit | Command::Show | Command::Add
        ) {
            return Ok(menu_command);
        }

        menu_command.path_id = parts
            .next()
            .ok_or("路径ID获取失败")?
            .parse::<u32>()
            .map_err(|_e| "路径ID解析失败".to_string())?;

        Ok(menu_command)
    }
}

fn save_configuration(config: &str) -> io::Result<()> {
    println!("{config}");
    fs::write("configuration.toml", config)
}

pub fn get_configuration(path: &str) -> Result<Vec<Configuration>, std::io::Error> {
    let mut configs: Vec<Configuration> = Vec::new();

    let file_content = fs::read_to_string(path).expect(&format!("找不到文件 {path}"));
    println!("{file_content}");
    let toml_value = file_content
        .parse::<Table>()
        .expect("Parse {path} file error");

    dbg!(&toml_value);

    for (host_name, configuration) in toml_value {
        // let mut java_configuration: Vec<JavaConfiguration>  =  Vec::new();

        let java_configuration: Vec<JavaConfiguration> = configuration["java"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| JavaConfiguration {
                name: x["name"].as_str().to_owned().unwrap().to_string(),
                path: x["path"].as_str().to_owned().unwrap().to_string(),
            })
            .collect();

        configs.push(Configuration {
            host_name,
            java_configuration,
            back_path: configuration["back_path"].to_string(),
        });
    }
    Ok(configs)
}

fn get_current_host_config(configs: &mut Vec<Configuration>) -> Option<&mut Configuration> {
    let host_name: String = gethostname()
        .into_string()
        .unwrap_or_else(|_e| "unkown_host".to_string());
    println!("Can't Get Current host name ,use unkown_host as concurrent host");

    if configs
        .iter_mut()
        .find(|config| config.host_name == host_name)
        .is_none()
    {
        configs.push(Configuration {
            host_name: gethostname()
                .into_string()
                .unwrap_or_else(|_e| "unkown_host".to_string()),
            ..Default::default()
        });
    }

    configs
        .iter_mut()
        .find(|config| config.host_name == host_name)
}

fn print_config(config: &Configuration) {
    let mut table = prettytable::Table::new();
    table.add_row(row!["HOST_NAME", config.host_name]);
    // table.add_row(row!["back_path", config.back_path]);
    for (idx, java_config) in config.java_configuration.iter().enumerate() {
        table.add_row(row![idx, java_config.name, java_config.path]);
    }

    table.printstd();
}

fn check_current_java() {
    let mut output = std::process::Command::new("java").arg("-version").output();
    match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stderr).into_owned();
            println!("当前JAVA版本是:{}", output.lines().next().unwrap());
        }
        Err(_e) => {
            println!("当前环境不存在JAVA程序");
            return;
        }
    }

    output = std::process::Command::new("where.exe")
        .arg("java.exe")
        .output();
    match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stdout).into_owned();
            println!("相关路径为:\n{}", &output);
        }
        Err(_e) => {
            println!("无法获得当前环境中JAVA程序相关路径");
            return;
        }
    }
}

fn check_java_version(path: &str) -> Result<String, String> {
    let bin_path = Path::new("./bin/java.exe");
    let path = Path::new(path).join(bin_path);
    let output = std::process::Command::new(path).arg("-version").output();
    match output {
        Ok(output) => {
            let output = String::from_utf8_lossy(&output.stderr).into_owned();
            let version = output
                .split_ascii_whitespace()
                .into_iter()
                .find(|s| s.starts_with("\"") && s.ends_with("\""))
                .ok_or("JAVA 版本解析失败")?
                .trim_matches('"');

            println!("JAVA 版本为: {}", &version);
            Ok(version.to_string())
        }
        Err(_e) => {
            println!("当前环境不存在JAVA程序");
            Err("JAVA 版本解析失败".to_string())
        }
    }
}

fn add_config(config: &mut Configuration) {
    check_current_java();
    println!("请输入新JDk路径:");
    let mut input_string = String::new();
    io::stdin()
        .read_line(&mut input_string)
        .expect("无法读取输入");
    input_string = input_string.trim().to_string();
    let java_path = input_string.clone();
    match check_java_version(&input_string) {
        Err(e) => {
            println!("{e}");
            return;
        }
        Ok(mut version) => {
            println!(
                "{}",
                format!("是否使用程序解析版本名称 {} 添加 (Y:Default/N)", &version)
            );
            input_string.clear();
            io::stdin()
                .read_line(&mut input_string)
                .expect("无法读取输入");
            input_string = input_string.trim().to_string().to_ascii_uppercase();
            dbg!(&input_string);

            if input_string == "N" {
                println!("请输入名称：");

                version.clear();
                io::stdin().read_line(&mut version).expect("无法读取输入");
            }

            config.java_configuration.push(JavaConfiguration {
                name: version,
                path: java_path,
            });

            println!("路径添加成功");
            print_config(config);
        }
    }
}

fn set_config(config: &mut Configuration, idx: usize) -> Result<String, io::Error> {
    let config = config.java_configuration.get(idx).unwrap();

    println!("aaaaaa");
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let reg_path: RegKey = hklm
        .open_subkey_with_flags(
            "SYSTEM\\CurrentControlSet\\Control\\Session Manager\\Environment",
            KEY_READ | KEY_WRITE,
        )
        .unwrap();
    let mut path = reg_path.get_value("Path").unwrap();
    let java_path = Path::new(&config.path).join("bin");
    path = format!(
        "{};{}",
        java_path.into_os_string().into_string().unwrap(),
        path
    );
    dbg!(&path);
    reg_path.set_value("Path", &path).unwrap();

    println!("添加环境变量{}成功", path);
    Ok("添加环境变量成功".to_string())
}

pub fn config_to_toml(configs: &Vec<Configuration>) -> String {
    let mut toml_table: toml::map::Map<String, toml::Value> = toml::Table::new();
    for config in configs {
        let mut config_table = toml::Table::new();
        config_table.insert(
            "back_path".to_string(),
            toml::Value::String(config.back_path.clone()),
        );
        let mut java_toml: Vec<toml::value::Value> = Vec::new();

        for java_config in &config.java_configuration {
            let mut java_table = toml::Table::new();
            java_table.insert(
                "name".to_string(),
                toml::Value::String(java_config.name.clone()),
            );
            java_table.insert(
                "path".to_string(),
                toml::Value::String(java_config.path.clone()),
            );
            java_toml.push(toml::Value::Table(java_table));
        }

        config_table.insert("java".to_string(), toml::Value::Array(java_toml));

        toml_table.insert(config.host_name.clone(), toml::Value::Table(config_table));
    }
    dbg!(&toml_table);
    toml_table.to_string()
}

fn change_config(config: &mut Configuration) {
    let mut input_string = String::new();
    loop {
        println!("\n=== 命令行菜单 ===");
        println!("A\t 添加");
        println!("C No \t 修改目标ID为当前环境变量");
        println!("D No \t 删除");
        println!("E  \t 退出");
        println!("S  \t 显示路径信息");
        println!("请输入选项编号:");

        input_string.clear();
        io::stdin()
            .read_line(&mut input_string)
            .expect("无法读取输入");

        let menu_command = MenuCommand::new(&input_string);

        match menu_command {
            Ok(menu_command) => match &menu_command.command {
                Command::Add => add_config(config),
                Command::Change => {
                    let java_configuration =
                        config.java_configuration.get(menu_command.path_id as usize);

                    if java_configuration.is_none() {
                        println!("路径ID {} 不存在", menu_command.path_id);
                        continue;
                    }
                    config.set_back();

                    set_config(config, menu_command.path_id as usize).unwrap();
                }
                Command::Del => print!("123"),
                Command::Show => print_config(config),
                Command::Exit => break,
            },
            Err(err) => {
                println!("{}", err);
                continue;
            }
        }
    }
}

pub fn core(configs: &mut Vec<Configuration>) {
    let config = get_current_host_config(configs).unwrap();
    print_config(config);
    change_config(config);
    save_configuration(&config_to_toml(configs)).unwrap();
}
