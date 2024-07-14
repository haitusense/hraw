use byteorder::{LittleEndian, WriteBytesExt};
use clap::Parser;
use colored::*;
use hraw::prelude::*;
use hraw::display::Indent;
use std::fs::File;

#[allow(unused)]
use std::io::{Write, Cursor};
#[allow(unused)]
use zip::{ZipArchive, write::ZipWriter};


/*** args ***/

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  args.run()?;
  Ok(())
}


/*** args ***/

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  #[command(subcommand)]
  pub command: SubCommands,
}
impl Args {
  fn run(&self) -> anyhow::Result<()> {
    match &self.command {
      SubCommands::Info(n) => n.run(),
      SubCommands::Convert(n) => n.run(),
      SubCommands::Accumulate(n) => n.run(),
    }
  }
}
#[derive(Debug, clap::Subcommand)]
enum SubCommands {
  Info(Info),
  Convert(Convert),
  Accumulate(Accumulate)
}


/******** subcommands ********/

trait Runner { fn run(&self) -> anyhow::Result<()>; }


/*** View ***/

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Info {
  input: Vec<String>,
}

impl Runner for Info {
  fn run(&self) -> anyhow::Result<()> {
    for i in &self.input {
      let hraw = hraw::Hraw::new(i.as_str())?;
      print_info(hraw)?;
    }
    Ok(())
  }
}

fn print_info(mut hraw: Hraw) -> anyhow::Result<()> {
  println!("{:<8}{}", "path".green().bold(), hraw.path_str().bold());

  println!("  {:<8}{}", "kind".green(), "path".green());
  println!("  {:<8}{}", "----".green(), "----".green());
  for (path, kind) in &hraw.info()? {
    println!("  {:<8}{}", kind, path);
  }

  println!("{:<8}{}", "header".green().bold(), hraw.header().headerpath.bold());
  let header = display::obj_to_yaml(hraw.header())?;
  println!("{}", header.indent(2));
  Ok(())
}


/*** Convert ***/

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Convert {

  input: Vec<String>,

  #[arg(short, long, default_value="out.hraw")]
  output: String,

  #[arg(short, long, default_value="1")]
  fixedpoint: usize,
}

impl Runner for Convert {
  fn run(&self) -> anyhow::Result<()> { 
    for i in &self.input {
      let hraw = Hraw::new(i.as_str())?;
      converter(hraw, self.output.as_str(), self.fixedpoint)?;
    }
    Ok(())
  }
}

fn converter(mut hraw: Hraw, output: &str, fixedpoint: usize) -> anyhow::Result<()> {
  println!("{}", hraw.path_str().green().bold());

  let header = hraw.header();
  let (width, height, size) = header.to_size();
  let list = header.data;
  let header = indoc::formatdoc! {"
    width : {width}
    height : {height}
    bitfield : unknown # le_f64
    data :
      - single.raw
      - mean.raw
      - std.raw
    decoder :
      lang : py
      code : |
        import struct
        def function(index):
          i = index * 8
          f = struct.unpack_from('<d', bytearray(src[i:i+8]), 0)[0]
          return int(f * {fixedpoint})
  "};

  let f = File::create(output)?; // let buf = Cursor::new(Vec::new());
  let mut zip = ZipWriter::new(f);
  let options = zip::write::FileOptions::default();

  // rms = root mean square
  // dev^2 = (rms)^2 - mean^2 
  let mut add = vec![0f64; size];
  let mut add_square = vec![0f64; size];

  zip.start_file("single.raw", options)?;
  for (i, subpath) in list.iter().enumerate() {
    println!("{} : {}", i, subpath);
    if i == 0 {
      for (index, data) in hraw.enumerate::<le_i32, _>(subpath.to_owned())? {
        zip.write_f64::<LittleEndian>(data as f64)?;
        add[index] += data as f64;
        add_square[index] += (data as f64).powf(2f64);
      }
    } else {
      for (index, data) in hraw.enumerate::<le_i32, _>(subpath.to_owned())? {
        add[index] += data as f64;
        add_square[index] += (data as f64).powf(2f64);
      }
    }
  }

  println!("  --> mean.raw");
  zip.start_file("mean.raw", options)?;
  for (_i, (add, _add_square)) in add.iter().zip(add_square.iter()).enumerate() {
    zip.write_f64::<LittleEndian>(add / list.len() as f64)?;
  }
  println!("  --> std.raw");
  zip.start_file("std.raw", options)?;
  for (_i, (add, add_square)) in add.iter().zip(add_square.iter()).enumerate() {
    zip.write_f64::<LittleEndian>(((add_square / list.len() as f64) - (add / list.len() as f64).powf(2f64)).sqrt())?;
  }  
  println!("  --> header.yaml");
  zip.start_file("header.yaml", options)?;

  zip.write(header.as_bytes())?;

  zip.flush()?;
  zip.finish()?;
  // merge_archive うごいてなくね？

  Ok(())
}


/*** Accumulate ***/

#[derive(clap::Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Accumulate {

  input: Vec<String>,

  #[arg(short, long)]
  script: String,

}

impl Runner for Accumulate {
  fn run(&self) -> anyhow::Result<()> { 
    for _i in &self.input {
      println!("{}", "Unimplemented".red().bold());
    }
    Ok(())
  }
}

// fn accumulate(mut hraw: Hraw) -> anyhow::Result<()> {
  // let header = hraw.header();
  // println!("{:#?}", header);

  // let list = header.to_subpath();
  // for i in &list {
  //   println!("{}", i);
  //   for (index, data) in hraw.enumerate_path::<le_i32>(i.as_str()) {
  //     println!("index : {}, data :{}", index, data)
  //   }
  // }

  // println!("{:?}", list);
//   Ok(())
// }