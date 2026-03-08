use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

#[derive(Clone, Copy, Debug)]
enum Language {
    Russian,
    English,
}

const EN_ALPHABET_UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const EN_ALPHABET_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";

const RU_ALPHABET_UPPER: &str = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯ";
const RU_ALPHABET_LOWER: &str = "абвгдеёжзийклмнопрстуфхцчшщъыьэюя";

fn main() -> io::Result<()> {
    let file_path = ask_string("Путь к исходному .txt файлу: ")?;
    let text = fs::read_to_string(&file_path)?;

    let language = ask_language()?;

    let shift: i32 = ask_shift("Смещение для шифра Цезаря: ")?;
    let encrypted = caesar_cipher(&text, shift, language);

    let output_path = "src/cipher_output.txt";
    fs::write(output_path, &encrypted)?;
    println!("Шифротекст сохранён в файл: {}", output_path);

    println!();
    println!("Частотный анализ исходного текста");
    let (src_counts, src_total) = count_letter_frequencies(&text, language);
    print_frequency_table(&src_counts, src_total, language);

    println!();
    println!("Частотный анализ шифротекста");
    let (enc_counts, enc_total) = count_letter_frequencies(&encrypted, language);
    print_frequency_table(&enc_counts, enc_total, language);

    println!();
    println!("Сравнение частот с табличной частотностью алфавита");
    // Для исходного текста просто сравниваем с теорией
    print_comparison_with_theoretical(&src_counts, src_total, language, "Исходный текст", false);
    println!();
    // Для шифротекста дополнительно выводим наиболее вероятный оригинальный символ
    print_comparison_with_theoretical(&enc_counts, enc_total, language, "Шифротекст", true);

    Ok(())
}

fn ask_string(prompt: &str) -> io::Result<String> {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn ask_shift(prompt: &str) -> io::Result<i32> {
    loop {
        let s = ask_string(prompt)?;
        match s.parse::<i32>() {
            Ok(v) => return Ok(v),
            Err(_) => {
                eprintln!("Ошибка: введите целое число.");
            }
        }
    }
}

fn ask_language() -> io::Result<Language> {
    loop {
        println!("Язык текста:");
        println!("1 - Русский");
        println!("2 - Английский");
        let choice = ask_string("")?;
        match choice.as_str() {
            "1" => return Ok(Language::Russian),
            "2" => return Ok(Language::English),
            _ => {
                eprintln!("Неверный выбор.");
            }
        }
    }
}

fn caesar_cipher(text: &str, shift: i32, lang: Language) -> String {
    text.chars()
        .map(|ch| shift_char(ch, shift, lang))
        .collect()
}

fn shift_char(ch: char, shift: i32, lang: Language) -> char {
    match lang {
        Language::English => shift_char_in_alphabet(ch, EN_ALPHABET_UPPER, EN_ALPHABET_LOWER, shift),
        Language::Russian => shift_char_in_alphabet(ch, RU_ALPHABET_UPPER, RU_ALPHABET_LOWER, shift),
    }
}

fn shift_char_in_alphabet(ch: char, upper: &str, lower: &str, shift: i32) -> char {
    if let Some(pos) = upper.chars().position(|c| c == ch) {
        let len = upper.chars().count() as i32;
        let new_pos = ((pos as i32 + shift).rem_euclid(len)) as usize;
        upper.chars().nth(new_pos).unwrap_or(ch)
    } else if let Some(pos) = lower.chars().position(|c| c == ch) {
        let len = lower.chars().count() as i32;
        let new_pos = ((pos as i32 + shift).rem_euclid(len)) as usize;
        lower.chars().nth(new_pos).unwrap_or(ch)
    } else {
        ch
    }
}

fn count_letter_frequencies(text: &str, lang: Language) -> (HashMap<char, usize>, usize) {
    let alphabet: Vec<char> = match lang {
        Language::English => EN_ALPHABET_LOWER.chars().collect(),
        Language::Russian => RU_ALPHABET_LOWER.chars().collect(),
    };

    let mut counts: HashMap<char, usize> = HashMap::new();
    let mut total_letters = 0usize;

    for ch in text.chars() {
        let ch_lower = ch.to_lowercase().next().unwrap_or(ch);
        if alphabet.contains(&ch_lower) {
            *counts.entry(ch_lower).or_insert(0) += 1;
            total_letters += 1;
        }
    }

    (counts, total_letters)
}

fn print_frequency_table(counts: &HashMap<char, usize>, total: usize, lang: Language) {
    if total == 0 {
        println!("В тексте не найдено букв выбранного алфавита.");
        return;
    }

    let mut items: Vec<(char, usize)> = counts.iter().map(|(&c, &n)| (c, n)).collect();
    items.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Символ | Кол-во | Процент");
    println!("---------------------------");
    for (ch, n) in items {
        let percent = (n as f64) * 100.0 / (total as f64);
        println!("{:>6} | {:>6} | {:>7.3} %", ch, n, percent);
    }

    println!("Всего букв ({}): {}", match lang { Language::Russian => "русский", Language::English => "английский" }, total);
}

fn print_comparison_with_theoretical(
    counts: &HashMap<char, usize>,
    total: usize,
    lang: Language,
    title: &str,
    show_probable_original: bool,
) {
    if total == 0 {
        println!("{}: нет букв.", title);
        return;
    }

    let (alphabet, theoretical) = match lang {
        Language::English => get_english_theoretical_frequencies(),
        Language::Russian => get_russian_theoretical_frequencies(),
    };

    println!("{}", title);
    if show_probable_original {
        println!("Буква | Набл.% | Теор.% | Разн.% | Вероятн. оригинал");
        println!("----------------------------------------------------");
    } else {
        println!("Буква | Набл.% | Теор.% | Разн.%");
        println!("---------------------------------");
    }

    let mapping = if show_probable_original {
        Some(build_probable_mapping(counts, total, lang))
    } else {
        None
    };

    for (i, ch) in alphabet.iter().enumerate() {
        let count = *counts.get(ch).unwrap_or(&0);
        let observed = (count as f64) * 100.0 / (total as f64);
        let expected = theoretical[i];
        let diff = observed - expected;
        if let Some(ref map) = mapping {
            let probable = map.get(ch).copied().unwrap_or(' ');
            println!(
                "{:>5} | {:>6.3} | {:>6.3} | {:+6.3} | {:>5}",
                ch, observed, expected, diff, probable
            );
        } else {
            println!(
                "{:>5} | {:>6.3} | {:>6.3} | {:+6.3}",
                ch, observed, expected, diff
            );
        }
    }
}

fn build_probable_mapping(
    counts: &HashMap<char, usize>,
    total: usize,
    lang: Language,
) -> HashMap<char, char> {
    let (alphabet, theoretical) = match lang {
        Language::English => get_english_theoretical_frequencies(),
        Language::Russian => get_russian_theoretical_frequencies(),
    };

    let n = alphabet.len();

    let mut observed: Vec<f64> = Vec::with_capacity(n);
    for ch in &alphabet {
        let count = *counts.get(ch).unwrap_or(&0);
        let freq = if total > 0 {
            (count as f64) * 100.0 / (total as f64)
        } else {
            0.0
        };
        observed.push(freq);
    }

    let mut idx_obs: Vec<usize> = (0..n).collect();
    idx_obs.sort_by(|&i, &j| observed[j].partial_cmp(&observed[i]).unwrap());

    let mut idx_theor: Vec<usize> = (0..n).collect();
    idx_theor.sort_by(|&i, &j| theoretical[j].partial_cmp(&theoretical[i]).unwrap());

    let mut map = HashMap::new();
    for k in 0..n {
        let cipher_ch = alphabet[idx_obs[k]];
        let plain_ch = alphabet[idx_theor[k]];
        map.insert(cipher_ch, plain_ch);
    }
    map
}

fn get_english_theoretical_frequencies() -> (Vec<char>, Vec<f64>) {
    let alphabet: Vec<char> = EN_ALPHABET_LOWER.chars().collect();
    let freqs = vec![
        8.167, // a
        1.492, // b
        2.782, // c
        4.253, // d
        12.702, // e
        2.228, // f
        2.015, // g
        6.094, // h
        6.966, // i
        0.153, // j
        0.772, // k
        4.025, // l
        2.406, // m
        6.749, // n
        7.507, // o
        1.929, // p
        0.095, // q
        5.987, // r
        6.327, // s
        9.056, // t
        2.758, // u
        0.978, // v
        2.360, // w
        0.150, // x
        1.974, // y
        0.074, // z
    ];
    (alphabet, freqs)
}

fn get_russian_theoretical_frequencies() -> (Vec<char>, Vec<f64>) {
    let alphabet: Vec<char> = RU_ALPHABET_LOWER.chars().collect();
    let freqs = vec![
        8.01,  // а
        1.59,  // б
        4.54,  // в
        1.70,  // г
        2.98,  // д
        8.45,  // е
        0.04,  // ё 
        0.94,  // ж
        1.65,  // з
        7.35,  // и
        1.21,  // й
        3.49,  // к
        4.40,  // л
        3.21,  // м
        6.70,  // н
        10.97, // о
        2.81,  // п
        4.73,  // р
        5.47,  // с
        6.26,  // т
        2.62,  // у
        0.26,  // ф
        0.97,  // х
        0.48,  // ц
        1.44,  // ч
        0.73,  // ш
        0.36,  // щ
        0.04,  // ъ
        1.90,  // ы
        1.74,  // ь
        0.32,  // э
        0.64,  // ю
        2.01,  // я
    ];
    (alphabet, freqs)
}

