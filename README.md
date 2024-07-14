# hraw

## description

Raw image format

## requirement

## installation

### lib

cargo.toml
```toml
[dependencies]
hraw = { git = "https://github.com/haitusense/hraw/", features=["pixel"] }

# install another version
hraw = { git = "https://github.com/haitusense/hraw/", rev = "bc7d76e" }
hraw = { git = "https://github.com/haitusense/hraw/", tag = "hoge" }
hraw = { git = "https://github.com/haitusense/hraw/", branch = "dev-0.1.3" }

# ! Not this one. (install from crates.io)
hraw = { git = "https://github.com/haitusense/hraw/", version = "1.0.0" }
```

### command line utilitie

```powershell
# local build
ps> cargo build --features app
# local run
ps> cargo run --features="app"

# install from github
ps> cargo install --git https://github.com/haitusense/hraw --features="app"
```

## Usage

### header structure

```yaml
ver      : "0.1"   # unused
width    : 1024    # [pixel]
height   : 768     # [pixel]
offset   : 64      # [byte]          default : 0 
bitfield : le_i32  # [enum BitField] default : le_i32
bayer: |           # unused
  R G
  G B
data :             # Optional [raw body path] default : ["data.raw"]
  - 1.raw
  - 2.raw
  - 3.raw
decoder :          # Optional use when bitfield = unknown
  lang : lua       # lua or py
  code : |
    function main(index)
      local i = index * 4 + 1
      local buf = buffer.create(4)
      buffer.writeu8(buf, 3, src[i + 3])
      buffer.writeu8(buf, 2, src[i + 2])
      buffer.writeu8(buf, 1, src[i + 1])
      buffer.writeu8(buf, 0, src[i + 0])
      local dst = bit32.rshift(buffer.readi32(buf, 0), 8)
      return dst
    end
```

### command line utilitie

```powershell
ps> hraw convert "data.hraw" -o "ave.hraw" --num 100 --fixed 100
```

data.hraw
```yaml
...
offset   : 64
bitfield : le_i32
data :
  - 202410100732.raw # timestamp + .raw
  - 202410100733.raw
  - 202410100734.raw
  ...
```

↓

ave.hraw
```yaml
...
bitfield : unknown   # le_f32 or unknown
data :
  - single_float.raw # 202410100732.raw cast to le_f32
  - ave_float.raw    # frames accumulation
  - dev_float.raw
decoder :
  lang : lua
  code : |
    function main(index)
      local i = index * 4 + 1;
      local a = bit32.lshift(src[i + 3], 24);
      local b = bit32.lshift(src[i + 2], 16);
      local c = bit32.lshift(src[i + 1], 8);
      local d = src[i + 0];
      local buf = bit32.bor(a, b, c, d);
      local dst = buffer.writef32(buf, 0, 0) * 100 --fixed point number x100
      return dst
    end
```

