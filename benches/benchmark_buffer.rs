// cargo bench --bench benchmark -- group1
// cargo bench benchmark_lua で実行

use criterion::{Criterion, criterion_group, criterion_main};
use byteorder::{LittleEndian, ReadBytesExt};


const TEST_FILE_I32 :&str = r".\src\tests\data\ship_i32.zip";

fn bench(c: &mut Criterion) {
  
  c.bench_function("zip read-bufread", |b| b.iter(|| {
    
    let mut zip = zip::ZipArchive::new(std::fs::File::open(TEST_FILE_I32).unwrap()).unwrap();
    let zf = zip.by_name("data.raw").unwrap();
    let mut reader = std::io::BufReader::new(zf);
    (0..640*480).for_each(|_i|{ 
      let _ = reader.read_i32::<LittleEndian>();
    });
  }));

  c.bench_function("zip read-read", |b| b.iter(|| {
    
    let mut zip = zip::ZipArchive::new(std::fs::File::open(TEST_FILE_I32).unwrap()).unwrap();
    let mut zf = zip.by_name("data.raw").unwrap();
    (0..640*480).for_each(|_i|{ 
      let _ = zf.read_i32::<LittleEndian>();
    });
  }));

  c.bench_function("zip bufread-read", |b| b.iter(|| {
    let file = std::fs::File::open(TEST_FILE_I32).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).unwrap();
    let mut zf = zip.by_name("data.raw").unwrap();
    (0..640*480).for_each(|_i|{ 
      let _ = zf.read_i32::<LittleEndian>();
    });
  }));

  c.bench_function("zip bufread-bufread", |b| b.iter(|| {
    let file = std::fs::File::open(TEST_FILE_I32).unwrap();
    let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).unwrap();
    let zf = zip.by_name("data.raw").unwrap();
    let mut reader = std::io::BufReader::new(zf);
    (0..640*480).for_each(|_i|{ 
      let _ = reader.read_i32::<LittleEndian>();
    });
  }));
}


criterion_group!(benches, bench);
criterion_main!(benches);