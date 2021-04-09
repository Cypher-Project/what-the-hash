use std::io::prelude::*; // File write_all

#[allow(dead_code)]
#[derive(serde::Deserialize)]
struct Mode {
  john: Option<String>,
  hashcat: Option<u64>,
  extended: bool,
  name: String,
}

#[derive(serde::Deserialize)]
struct Regex {
  regex: String,
  modes: Vec<Mode>,
}

fn download(filename: &str) -> Result<String, Box<dyn std::error::Error>> {
  let remote = format!("https://www.onlinehashcrack.com/static/{}", filename);
  let body = reqwest::blocking::get(remote)?.text()?;
  Ok(body)
}

fn fwrite(filename: &str, contents: String) -> Result<(), Box<dyn std::error::Error>> {
  let mut file = std::fs::File::create(filename)?;
  file.write_all(contents.as_bytes())?;
  Ok(())
}

fn jread(filename: &str) -> Result<Vec<Regex>, Box<dyn std::error::Error>> {
  let file = std::fs::File::open(filename)?;
  let reader = std::io::BufReader::new(file);
  let u = serde_json::from_reader(reader)?;
  Ok(u)
}

fn cache(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
  let data = match download(filename) {
    Ok(data) => data,
    Err(err) => {
      println!("Cannot download {}", filename);
      return Err(err)  
    }
  };

  match fwrite(filename, data) {
    Ok(_) => (),
    Err(err) => {
      println!("Cannot save {}", filename);
      return Err(err)
    }
  }

  Ok(())
}

fn display_match(modes : Vec<Mode>) {
  for mode in modes {
    println!("[*] {}", mode.name);
    if mode.john.is_some() {
      println!("    john format:  {}", mode.john.unwrap());
    }

    if mode.hashcat.is_some() {
      println!("    hashcat mode: {}", mode.hashcat.unwrap());
    }
  } 
}

fn get_hash() -> Result<String, Box<dyn std::error::Error>> {
  let args : Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    Err("missing argument (filepath or hash)")?
  }

  let arg : String = std::env::args().nth(1).unwrap();
  if std::path::Path::new(&arg).exists() {
    Ok(std::fs::read_to_string(&arg)?.trim_end().to_string())
  } else {
    Ok(arg)
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let hash = get_hash()?;
  println!("Trying to identify {}", hash);
  let path = "hash-id-prototypes.json";
  if ! std::path::Path::new(path).exists() {
    cache(path)?;
  }

  let regexps = jread(path)?;
  for regex in regexps {
    let re = regress::Regex::new(&regex.regex);
    if re.is_err() {
      //eprintln!("Cannot use '{}', skipping", regex.regex);
      continue;
    }

    if ! re.unwrap().find(&hash).is_some() {
      continue;
    }

    display_match(regex.modes);
  }

  Ok(())
}
