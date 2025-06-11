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
    // println!("{text}");
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
    let family = &target.chips.first().unwrap().family.to_lowercase();
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
        print_and_write(&mut file, "  flash_algorithms:")?;
        print_and_write(&mut file, &format!("  - {}", family))?;
        // println!("");
    }
    print_and_write(&mut file, "flash_algorithms:")?;
    print_and_write(&mut file, &format!("- name: {}", family))?;
    print_and_write(&mut file, &format!("  description: {}", "generic algo for family"))?;
    print_and_write(&mut file, &format!("  default: {}", "true"))?;
    print_and_write(&mut file, &format!("  instructions: {}", "AAAAAAAA"))?;
    print_and_write(&mut file, r#"  pc_init: 0x1
  pc_uninit: 0xbd
  pc_program_page: 0x1cd
  pc_erase_sector: 0xd1
  data_section_offset: 0x10000c24
  rtt_location: 0x10000bec
  flash_properties:
    address_range:
      start: 0x0
      end: 0x80000
    page_size: 0x400
    erased_byte_value: 0xff
    program_page_timeout: 1500
    erase_sector_timeout: 1500
    sectors:
    - size: 0x1000
      address: 0x0
    - size: 0x8000
      address: 0x10000
  cores:
  - main
    "#)?;

    Ok(())
}

/*
flash_algorithms:
  - lpc17xx
flash_algorithms:
- name: lpc17xx
  description: A flash algorithm under test
  default: true
  instructions: 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
  pc_init: 0x1
  pc_uninit: 0xbd
  pc_program_page: 0x1cd
  pc_erase_sector: 0xd1
  data_section_offset: 0x10000c24
  rtt_location: 0x10000bec
  flash_properties:
    address_range:
      start: 0x0
      end: 0x80000
    page_size: 0x400
    erased_byte_value: 0xff
    program_page_timeout: 1500
    erase_sector_timeout: 1500
    sectors:
    - size: 0x1000
      address: 0x0
    - size: 0x8000
      address: 0x10000
  cores:
  - main

*/