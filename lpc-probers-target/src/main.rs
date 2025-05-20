use std::{fs::File, io::BufReader};

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

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let file = File::open(&args.target_json_filename)?;
    let reader = BufReader::new(file);
    let target: Chips = serde_json::from_reader(reader)?;
    println!("File {}", &args.target_json_filename);
    // println!("Target: {target:?}");
    println!();
    let family = &target.chips.first().unwrap().family;
    let coretype = "armv6m";

    println!("name: {}", family);
    println!("varients:");
    for chip in target.chips {
        // println!("{:?}", chip);
        println!("- name: {}", chip.id);
        println!("  cores:");
        println!("  - name: main");
        println!("    type: {}", coretype);
        println!("    core_access_options: !Arm");
        println!("      ap: 0");
        println!("      psel: 0x0");
        println!("  memory_map:");

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
            println!("  - !{}", typestr);
            println!("    name: {}", entry.name);
            println!("    range:");
            println!("      start: 0x{:X}", entry.start);
            println!("      end: 0x{:X}", entry.end);
            println!("    cores:");
            for core in entry.cores {
                println!("      - {}", core);
            }
            // println!("{:?}", entry);
        }
        // println!("");
    }
    Ok(())
}
