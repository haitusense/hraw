# hraw

Raw image format

## Description

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
decoder : |        # Optional use when bitfield = unknown, 
  function(index)
    local i = index * 4 + 1; --index starts with 1
    local a = bit32.lshift(src[i + 3], 24);
    local b = bit32.lshift(src[i + 2], 16);
    local c = bit32.lshift(src[i + 1], 8);
    local d = src[i + 0];
    return bit32.bor(a, b, c, d);
  end
```

### grammar of decoder

with luau

- [luau-lang.org](https://luau-lang.org/)
- [Online Lua Compiler](https://luau-lang.org/demo)

**Builtin types**

- nil
- string
- number : 64-bit IEEE754 double precision number
- boolean
- table
- function
- thread
- userdata
- buffer
- any

```lua
--!strict

function(index : number) --number = 64-bit
  local i = index * 4 + 1; --index starts with 1
  local a = bit32.lshift(0xFF, 24)
  local b = bit32.lshift(0xAF, 16)
  local c = bit32.lshift(0x45, 8)
  local d = bit32.lshift(0x32, 0)

  local value1 = bit32.bor(a, b, c, d)
  print(string.format("0x%016X", value1)) -- 0x00000000FFAF4532
  if value1 > 0x7FFF_FFFF then
    value1 = -(0xFFFF_FFFF - value1 + 1)  
  end
  print(string.format("0x%016X", value1)) -- 0xFFFFFFFFFFAF4532
  
  local value2 = bit32.bor(a, b, c, vadl4)
  value2 = bit32.lshift(value2, 12)
  print(string.format("0x%016X", value2)) -- 0x00000000F4532000 / 32bit bitshift
  value2 = bit32.rshift(value2, 12)
  print(string.format("0x%016X", value2)) -- 0x00000000000F4532 / 32bit unsigned right shift

  local buf = buffer.create(4)            -- create 4byte buffer
  print(string.format("len %d", buffer.len(buf))) 
  buffer.writeu8(buf, 3, 0xFF)            -- write to buffer (little endian)
  buffer.writeu8(buf, 2, 0xAF)
  buffer.writeu8(buf, 1, 0x45)
  buffer.writeu8(buf, 0, 0x32)

  print( string.format("i32 %d", buffer.readi32(buf, 0)) ) -- i32 -5290702         / signed 32-bit integer
  print( string.format("u32 %d", buffer.readu32(buf, 0)) ) -- u32 4289676594       / unsigned 32-bit integer
  print( string.format("0x%X", buffer.readi32(buf, 0)) )   -- 0xFFFFFFFFFFAF4532
  print( string.format("%d", buffer.readf32(buf, 0)) )     -- -9223372036854775808 / 32-bit floating-point number

  return value1;                          -- -5290702
end
```

