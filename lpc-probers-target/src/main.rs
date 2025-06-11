use std::{
    fs::File,
    io::{BufReader, Write},
};

use clap::Parser;

use serde::Deserialize;

/// Tool to convert LPC targets to probe-rs yaml target files
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// target json to read
    target_json_filename: String,
}

#[derive(Deserialize, Debug)]
struct MemoryLoc {
    id: String,
    derived_from: String,
    location: String,
    size: String,
}

#[derive(Deserialize, Debug)]
struct Chip {
    id: String,
    family: String,
    memory: Vec<MemoryLoc>,
}

#[derive(Deserialize, Debug)]
struct Chips {
    chips: Vec<Chip>,
}

#[derive(Deserialize, Debug)]
enum MemoryType {
    Nvm,
    Ram,
}

#[derive(Deserialize, Debug)]
struct MemoryRange {
    name: String,
    start: usize,
    end: usize,
    cores: Vec<String>,
    memtype: MemoryType,
}

fn print_and_write(file: &mut File, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("{text}");
    file.write_all(text.as_bytes())?;
    file.write_all("\n".as_bytes())?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    process_file().unwrap();
    Ok(())
}

fn process_file() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let file = File::open(&args.target_json_filename)?;
    let reader = BufReader::new(file);
    let mut target: Chips = serde_json::from_reader(reader)?;
    println!("File {}", &args.target_json_filename);
    // println!("Target: {target:?}");
    println!();
    let family = &target.chips.first().unwrap().family;
    let coretype = "armv6m";
    let mut file = File::create(format!("./{family}_generated.yaml"))?;
    print_and_write(&mut file, &format!("name: {}", family))?;
    print_and_write(&mut file, "variants:")?;

    target.chips.sort_by(|a, b| a.id.cmp(&b.id));
    for chip in target.chips {
        // println!("{:?}", chip);
        print_and_write(&mut file, &format!("- name: {}", chip.id))?;
        print_and_write(&mut file, "  cores:")?;
        print_and_write(&mut file, "  - name: main")?;
        print_and_write(&mut file, &format!("    type: {}", coretype))?;
        print_and_write(&mut file, "    core_access_options: !Arm")?;
        print_and_write(&mut file, "      ap: !v1 0")?;
        print_and_write(&mut file, "  memory_map:")?;

        let mut nvm: Vec<MemoryRange> = vec![];
        for memory_range in chip.memory {
            // println!("{:?}", memory_range);
            // println!("[{}]", &memory_range.location);
            let memory_start =
                usize::from_str_radix(memory_range.location.trim_start_matches("0x"), 16).unwrap();
            let memory_size =
                usize::from_str_radix(memory_range.size.trim_start_matches("0x"), 16).unwrap();
            let memory_end = memory_start + memory_size;
            let memtype = memory_range.derived_from.as_str();
            match memtype {
                "Flash" => {
                    nvm.push(MemoryRange {
                        name: memory_range.id,
                        start: memory_start,
                        end: memory_end,
                        cores: vec!["main".into()],
                        memtype: MemoryType::Nvm,
                    });
                }
                "RAM" => {
                    nvm.push(MemoryRange {
                        name: memory_range.id,
                        start: memory_start,
                        end: memory_end,
                        cores: vec!["main".into()],
                        memtype: MemoryType::Ram,
                    });
                }
                _ => {
                    println!("## unhandled memory type {}", memtype);
                }
            };
        }
        for entry in nvm {
            let typestr = match entry.memtype {
                MemoryType::Nvm => "Nvm",
                MemoryType::Ram => "Ram",
            };
            print_and_write(&mut file, &format!("  - !{}", typestr))?;
            print_and_write(&mut file, &format!("    name: {}", entry.name))?;
            print_and_write(&mut file, "    range:")?;
            print_and_write(&mut file, &format!("      start: 0x{:X}", entry.start))?;
            print_and_write(&mut file, &format!("      end: 0x{:X}", entry.end))?;
            print_and_write(&mut file, "    cores:")?;
            for core in entry.cores {
                print_and_write(&mut file, &format!("      - {}", core))?;
            }
            // println!("{:?}", entry);
        }
        // println!("");
    }
    Ok(())
}
