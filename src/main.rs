mod color;
mod data;

use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::{io, thread};
use std::io::{BufRead, BufReader, stdout, Write};
use std::path::Path;
use std::time::{Duration, Instant};
use crate::color::{blue, cyan, green, magenta, red};
use sha2::{Digest, Sha256};
use base58::{FromBase58, ToBase58};
use rand::Rng;
use sv::util::{hash160};

const BACKSPACE: char = 8u8 as char;
const FILE_CONFIG: &str = "confPUBLIC.txt";

#[tokio::main]
async fn main() {
    let version: &str = env!("CARGO_PKG_VERSION");
    println!("{}", blue("==================="));
    println!("{}{}", blue("FIND PUBLIC KEY v:"), magenta(version));
    println!("{}", blue("==================="));

    //Чтение настроек, и если их нет создадим
    //-----------------------------------------------------------------
    let conf = match lines_from_file(&FILE_CONFIG) {
        Ok(text) => { text }
        Err(_) => {
            add_v_file(&FILE_CONFIG, data::get_conf_text().to_string());
            lines_from_file(&FILE_CONFIG).unwrap()
        }
    };

    //количество ядер процессора
    let count_cpu = num_cpus::get();
    let cpu_core: usize = first_word(&conf[0].to_string()).to_string().parse::<usize>().unwrap();
    let start_perebor = first_word(&conf[1].to_string()).to_string();
    let next_rand: usize = first_word(&conf[2].to_string()).to_string().parse::<usize>().unwrap();
    //---------------------------------------------------------------------

    //читаем файл с адресами и конвертируем их в h160 для базы
    //-----------------------------------------------------------------
    let file_content = match lines_from_file("puzzle.txt") {
        Ok(file) => { file }
        Err(_) => {
            let dockerfile = include_str!("puzzle.txt");
            add_v_file("puzzle.txt", dockerfile.to_string());
            lines_from_file("puzzle.txt").expect("kakoyto_pizdec")
        }
    };

    //хешируем
    let mut database = HashSet::new();
    for address in file_content.iter() {
        let binding = address.from_base58().unwrap();
        // Создание пустого массива [u8; 20]
        let mut a: [u8; 20] = [0; 20];
        // Копирование элементов из среза в массив фиксированного размера
        a.copy_from_slice(&binding.as_slice()[1..21]);
        database.insert(a);
    }

    //-----------------------------------------------------------------------
    println!("{}{}{}", blue("КОЛИЧЕСТВО ЯДЕР ПРОЦЕССОРА:"), green(cpu_core), blue(format!("/{count_cpu}")));
    println!("{}{}", blue("АДРЕСОВ ЗАГРУЖЕННО:"), green(database.len()));
    println!("{}{}", blue("НАЧАЛО ПЕРЕБОРА:"), green(start_perebor.clone()));
    println!("{}{}{}", blue("РАНДОМ КАЖДЫЕ:"), green(next_rand.clone()),blue(" секунд"));

    //для измерения скорости
    let mut start = Instant::now();
    let mut speed: u32 = 0;

    let mut rng = rand::thread_rng();

    let mut password = String::with_capacity(66); // Предварительное выделение памяти для строки
    password.push_str(&start_perebor);
    let dlinn_a_pasvord = 66;

    let alvabet="0123456789abcdef".to_string();

    let charset_chars: Vec<char> = alvabet.chars().collect();
    let charset_len = charset_chars.len(); //16
    //состовляем начальную позицию
    let mut current_combination = vec![0; dlinn_a_pasvord];
    for d in 0..dlinn_a_pasvord {
        let rand_w = rng.gen_range(0..charset_len);
        let position = charset_chars.iter().position(|&ch| ch == start_perebor.chars().nth(d).unwrap_or(charset_chars[rand_w])).unwrap();
        current_combination[d] = position;
    };

    let mut rerandom = 0;


    // if password.chars().count() > 66{
    //     println!("{}{}", red("КОЛИЧЕСТВО СИМВОЛОВ ДОЛЖНО БЫТЬ 66 != "),password.chars().count());
    //     jdem_user_to_close_programm();
    //     return;
    // }

    loop {
        // следующая комбинация пароля
        let password_string = String::from_iter(
            current_combination.iter().map(|&idx| charset_chars[idx])
        );

        // Разбиваем шестнадцатеричную строку на части по два символа
        let byte_public: Vec<u8> = password_string.as_str()
            .as_bytes()
            .chunks(2)
            .map(|chunk| u8::from_str_radix(&String::from_utf8_lossy(chunk), 16).unwrap())
            .collect();


        //проверяем есть ли в базе
        if database.contains(&hash160(&byte_public).0) {
            let address = get_legacy(hash160(&byte_public).0, 0x00);
            print_and_save(hex::encode(&byte_public), address);
        }

        let mut i = dlinn_a_pasvord;
        while i > 0 {
            i -= 1;
            if current_combination[i] + 1 < charset_len {
                current_combination[i] += 1;
                break;
            } else {
                current_combination[i] = 0;
            }
        }

        //измеряем скорость и шлём прогресс
        speed = speed + 1;
        if start.elapsed() >= Duration::from_secs(1) {

            let mut stdout = stdout();
            let hhh = hex::encode(byte_public);
            print!("{}\r{}", BACKSPACE, green(format!("SPEED:{speed}/s|{}", hhh)));
            stdout.flush().unwrap();
            start = Instant::now();
            speed = 0;

            rerandom +=1;
            //обновим
            if rerandom >next_rand{
                current_combination = vec![0; dlinn_a_pasvord];
                for d in 0..dlinn_a_pasvord {
                    let rand_w = rng.gen_range(0..charset_len);
                    let position = charset_chars.iter().position(|&ch| ch == start_perebor.chars().nth(d).unwrap_or(charset_chars[rand_w])).unwrap();
                    current_combination[d] = position;
                };
                rerandom=0;
            }
            //

        }
    }
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn add_v_file(name: &str, data: String) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("cannot open file")
        .write(data.as_bytes())
        .expect("write failed");
}

fn print_and_save(hex: String, addres: String) {
    println!("{}", cyan("\n!!!!!!!!!!!!!!!!!!!!!!FOUND!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"));
    println!("{}{}", cyan("PUBLIC KEY:"), cyan(hex.clone()));
    println!("{}{}", cyan("ADDRESS:"), cyan(addres.clone()));
    let s = format!("PUBLIC KEY:{}\nADDRESS {}\n", hex, addres);
    add_v_file("FOUND.txt", s);
    println!("{}", cyan("SAVE TO FOUND.txt"));
    println!("{}", cyan("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"));
}

fn sha256d(data: &[u8]) -> Vec<u8> {
    let first_hash = Sha256::digest(data);
    let second_hash = Sha256::digest(first_hash);
    second_hash.to_vec()
}

pub fn get_legacy(hash160: [u8; 20], coin: u8) -> String {
    let mut v = Vec::with_capacity(23);
    v.push(coin);
    v.extend_from_slice(&hash160);
    let checksum = sha256d(&v);
    v.extend_from_slice(&checksum[0..4]);
    let b: &[u8] = v.as_ref();
    b.to_base58()
}

fn first_word(s: &String) -> &str {
    s.trim().split_whitespace().next().unwrap_or("")
}

fn jdem_user_to_close_programm() {
    // Ожидание ввода пользователя для завершения программы
    println!("{}", blue("Нажмите Enter, чтобы завершить программу..."));
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Ошибка чтения строки");
}